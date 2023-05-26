use std::collections::BTreeMap;
use std::time::Duration;

use rand::prelude::*;
use serde::Serialize;
use snarker::job_commitment::JobCommitmentsConfig;
use tokio::select;
use tokio::sync::{mpsc, oneshot};

use snarker::account::AccountSecretKey;
use snarker::event_source::{
    Event, EventSourceProcessEventsAction, EventSourceWaitForEventsAction,
    EventSourceWaitTimeoutAction,
};
use snarker::p2p::channels::ChannelId;
use snarker::p2p::connection::outgoing::P2pConnectionOutgoingInitOpts;
use snarker::p2p::identity::SecretKey;
use snarker::p2p::service_impl::webrtc_rs::{Cmd, P2pServiceCtx, P2pServiceWebrtcRs, PeerState};
use snarker::p2p::{P2pConfig, P2pEvent, PeerId};
use snarker::rpc::RpcRequest;
use snarker::service::{EventSourceService, Stats};
use snarker::{Config, SnarkerConfig, State};

mod http_server;

mod rpc;
use rpc::RpcP2pConnectionOutgoingResponse;

#[derive(Debug, structopt::StructOpt)]
#[structopt(name = "snarker", about = "Openmina snarker")]
pub struct Snarker {
    #[structopt(long, default_value = "3000")]
    pub http_port: u16,
}

impl Snarker {
    pub fn run(self) -> Result<(), crate::CommandError> {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(3)
            .build()
            .unwrap();
        let _rt_guard = rt.enter();
        let local_set = tokio::task::LocalSet::new();
        let _local_set_guard = local_set.enter();

        let mut rng = ThreadRng::default();
        let bytes = rng.gen();
        let secret_key = SecretKey::from_bytes(bytes);
        let pub_key = secret_key.public_key();
        let peer_id = PeerId::from_public_key(pub_key.clone());
        eprintln!("peer_id: {peer_id}");

        let sec_key: AccountSecretKey = match std::env::var("MINA_SNARKER_SEC_KEY") {
            Ok(v) => match v.parse() {
                Err(err) => {
                    return Err(format!("error while parsing `MINA_SNARKER_SEC_KEY`: {err}").into())
                }
                Ok(v) => v,
            },
            Err(err) => {
                return Err(format!("env `MINA_SNARKER_SEC_KEY` not set! {err}").into());
            }
        };

        let config = Config {
            snarker: SnarkerConfig {
                public_key: sec_key.public_key(),
                job_commitments: JobCommitmentsConfig {
                    commitment_timeout: Duration::from_secs(6 * 60),
                },
            },
            p2p: P2pConfig {
                identity_pub_key: pub_key,
                initial_peers: vec![
                    "/2cFXX6RK81MUi7Fe4bFx7k3U6Fr1Hn1rDDbAqPEm6iUTdv2C2gw/http/localhost/3000"
                        .parse()
                        .unwrap(),
                ],
                max_peers: 10,
                enabled_channels: [ChannelId::SnarkJobCommitmentPropagation].into(),
            },
        };
        let state = State::new(config);
        let P2pServiceCtx { cmd_sender, peers } = SnarkerService::init();

        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        let (p2p_event_sender, mut rx) = mpsc::unbounded_channel::<P2pEvent>();
        let ev_sender = event_sender.clone();
        tokio::spawn(async move {
            while let Some(v) = rx.recv().await {
                if let Err(_) = ev_sender.send(v.into()) {
                    break;
                }
            }
        });

        let mut service = SnarkerService {
            rng,
            event_sender,
            p2p_event_sender,
            event_receiver: event_receiver.into(),
            cmd_sender,
            peers,
            rpc: rpc::RpcService::new(),
            stats: Stats::new(),
        };

        let http_port = self.http_port;
        let rpc_sender = service.rpc_req_sender().clone();
        let rpc_sender = RpcSender { tx: rpc_sender };
        // http-server
        // TODO(binier): separate somehow so that http server tasks could
        // never take resources from the state machine thread. Maybe
        // below code already does that.
        rt.spawn_blocking(move || {
            let local_set = tokio::task::LocalSet::new();
            let main_fut = local_set.run_until(http_server::run(http_port, rpc_sender));
            tokio::runtime::Handle::current().block_on(main_fut);
        });

        let mut snarker = ::snarker::Snarker::new(state, service);

        snarker
            .store_mut()
            .dispatch(EventSourceProcessEventsAction {});
        rt.block_on(async {
            loop {
                snarker
                    .store_mut()
                    .dispatch(EventSourceWaitForEventsAction {});

                let service = &mut snarker.store_mut().service;
                let wait_for_events = service.event_receiver.wait_for_events();
                let rpc_req_fut = async {
                    // TODO(binier): optimize maybe to not check it all the time.
                    match service.rpc.req_receiver().recv().await {
                        Some(v) => v,
                        None => std::future::pending().await,
                    }
                };
                let timeout = tokio::time::sleep(Duration::from_millis(1000));

                select! {
                    _ = wait_for_events => {
                        while snarker.store_mut().service.event_receiver.has_next() {
                            snarker.store_mut().dispatch(EventSourceProcessEventsAction {});
                        }
                    }
                    req = rpc_req_fut => {
                        snarker.store_mut().service.process_rpc_request(req);
                    }
                    _ = timeout => {
                        snarker.store_mut().dispatch(EventSourceWaitTimeoutAction {});
                    }
                }
            }
        });

        Ok(())
    }
}

struct SnarkerService {
    rng: ThreadRng,
    event_sender: mpsc::UnboundedSender<Event>,
    // TODO(binier): change so that we only have `event_sender`.
    p2p_event_sender: mpsc::UnboundedSender<P2pEvent>,
    event_receiver: EventReceiver,
    cmd_sender: mpsc::UnboundedSender<Cmd>,
    peers: BTreeMap<PeerId, PeerState>,
    rpc: rpc::RpcService,
    stats: Stats,
}

impl redux::TimeService for SnarkerService {}

impl redux::Service for SnarkerService {}

impl snarker::Service for SnarkerService {
    fn stats(&mut self) -> Option<&mut Stats> {
        Some(&mut self.stats)
    }
}

impl EventSourceService for SnarkerService {
    fn next_event(&mut self) -> Option<Event> {
        self.event_receiver.try_next()
    }
}

impl P2pServiceWebrtcRs for SnarkerService {
    fn random_pick(
        &mut self,
        list: &[P2pConnectionOutgoingInitOpts],
    ) -> P2pConnectionOutgoingInitOpts {
        list.choose(&mut self.rng).unwrap().clone()
    }

    fn event_sender(&mut self) -> &mut mpsc::UnboundedSender<P2pEvent> {
        &mut self.p2p_event_sender
    }

    fn cmd_sender(&mut self) -> &mut mpsc::UnboundedSender<Cmd> {
        &mut self.cmd_sender
    }

    fn peers(&mut self) -> &mut BTreeMap<PeerId, PeerState> {
        &mut self.peers
    }
}

pub struct EventReceiver {
    rx: mpsc::UnboundedReceiver<Event>,
    queue: Vec<Event>,
}

impl EventReceiver {
    /// If `Err(())`, `mpsc::Sender` for this channel was dropped.
    pub async fn wait_for_events(&mut self) -> Result<(), ()> {
        let next = self.rx.recv().await.ok_or(())?;
        self.queue.push(next);
        Ok(())
    }

    pub fn has_next(&mut self) -> bool {
        if self.queue.is_empty() {
            if let Some(event) = self.try_next() {
                self.queue.push(event);
                true
            } else {
                false
            }
        } else {
            true
        }
    }

    pub fn try_next(&mut self) -> Option<Event> {
        if !self.queue.is_empty() {
            Some(self.queue.remove(0))
        } else {
            self.rx.try_recv().ok()
        }
    }
}

impl From<mpsc::UnboundedReceiver<Event>> for EventReceiver {
    fn from(rx: mpsc::UnboundedReceiver<Event>) -> Self {
        Self {
            rx,
            queue: Vec::with_capacity(1),
        }
    }
}

pub struct SnarkerRpcRequest {
    pub req: RpcRequest,
    pub responder: Box<dyn Send + std::any::Any>,
}

#[derive(Clone)]
pub struct RpcSender {
    tx: mpsc::Sender<SnarkerRpcRequest>,
}

impl RpcSender {
    pub fn new(tx: mpsc::Sender<SnarkerRpcRequest>) -> Self {
        Self { tx }
    }

    pub async fn oneshot_request<T>(&self, req: RpcRequest) -> Option<T>
    where
        T: 'static + Send + Serialize,
    {
        let (tx, rx) = oneshot::channel::<T>();
        let responder = Box::new(tx);
        let sender = self.tx.clone();
        let _ = sender.send(SnarkerRpcRequest { req, responder }).await;

        rx.await.ok()
    }

    pub async fn multishot_request<T>(
        &self,
        expected_messages: usize,
        req: RpcRequest,
    ) -> mpsc::Receiver<T>
    where
        T: 'static + Send + Serialize,
    {
        let (tx, rx) = mpsc::channel::<T>(expected_messages);
        let responder = Box::new(tx);
        let sender = self.tx.clone();
        let _ = sender.send(SnarkerRpcRequest { req, responder }).await;

        rx
    }

    pub async fn peer_connect(
        &self,
        opts: P2pConnectionOutgoingInitOpts,
    ) -> Result<String, String> {
        let peer_id = opts.peer_id;
        let req = RpcRequest::P2pConnectionOutgoing(opts);
        self.oneshot_request::<RpcP2pConnectionOutgoingResponse>(req)
            .await
            .ok_or_else(|| "state machine shut down".to_owned())??;

        Ok(peer_id.to_string())
    }
}