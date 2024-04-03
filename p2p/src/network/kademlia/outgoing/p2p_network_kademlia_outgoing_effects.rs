use redux::ActionMeta;

use crate::{
    connection::outgoing::{
        P2pConnectionOutgoingAction, P2pConnectionOutgoingInitLibp2pOpts,
        P2pConnectionOutgoingInitOpts,
    },
    messages::find_node_request,
    token::{DiscoveryAlgorithm, StreamKind},
    webrtc::Host,
    Data, P2pNetworkConnectionMuxState, P2pNetworkKademliaError, P2pNetworkYamuxOpenStreamAction,
    P2pNetworkYamuxOutgoingDataAction, P2pStore,
};

use super::p2p_network_kademlia_outgoing_actions::P2pNetworkKademliaOutgoingAction;

impl P2pNetworkKademliaOutgoingAction {
    pub fn effects<Store, S>(self, meta: &ActionMeta, store: &mut Store)
    where
        Store: P2pStore<S>,
    {
        let config = &store.state().network.scheduler.discovery_state.config;

        match self {
            Self::ConnectInit {
                id,
                peer_id,
                address,
            } => {
                let host = match address {
                    std::net::SocketAddr::V4(ip) => Host::Ipv4(*ip.ip()),
                    std::net::SocketAddr::V6(ip) => Host::Ipv6(*ip.ip()),
                };

                let libp2p_opts = P2pConnectionOutgoingInitLibp2pOpts {
                    peer_id,
                    host,
                    port: address.port(),
                };
                let opts = P2pConnectionOutgoingInitOpts::LibP2P(libp2p_opts);

                store.dispatch(P2pConnectionOutgoingAction::Init { opts, rpc_id: None });
                store.dispatch(Self::ConnectPending {
                    id,
                    peer_id,
                    address,
                    time: meta.time(),
                });
            }
            Self::ConnectPending {
                id,
                peer_id,
                address,
                time,
            } => {
                if let Some(connection) = store.state().network.scheduler.connections.get(&address)
                {
                    if let Some(stream) = connection.mux.as_ref() {
                        let P2pNetworkConnectionMuxState::Yamux(stream) = stream;
                        if stream.init {
                            let Some(stream_id) = stream.streams.keys().max() else {
                                return;
                            };

                            let stream_id = (*stream_id).max(2);

                            store.dispatch(Self::ConnectSuccess {
                                id,
                                peer_id,
                                address,
                                stream_id: stream_id + 1,
                            });
                            return;
                        }
                    }
                } else {
                    // store.dispatch(self);
                }

                if let Some(time) = meta.time().checked_sub(time) {
                    if &time > config.timeout() {
                        store.dispatch(Self::ConnectFailed {
                            id,
                            peer_id,
                            address,
                            error: P2pNetworkKademliaError::Timeout,
                        });
                    }
                }
            }
            Self::ConnectSuccess {
                id,
                stream_id,
                peer_id,
                address,
            } => {
                store.dispatch(Self::StreamInit {
                    id,
                    stream_id,
                    peer_id,
                    address,
                });
            }
            Self::StreamInit {
                id,
                stream_id,
                peer_id,
                address,
            } => {
                if store.dispatch(P2pNetworkYamuxOpenStreamAction {
                    addr: address,
                    stream_id,
                    stream_kind: StreamKind::Discovery(DiscoveryAlgorithm::Kademlia1_0_0),
                }) {
                    store.dispatch(Self::StreamPending {
                        id,
                        stream_id,
                        peer_id,
                        address,
                        time: meta.time(),
                    });
                } else {
                    todo!()
                };
            }
            Self::StreamPending {
                time,
                stream_id,
                peer_id,
                address,
                id,
            } => {
                if let Some(time) = meta.time().checked_sub(time) {
                    if &time > config.timeout() {
                        store.dispatch(Self::StreamFailed {
                            id,
                            peer_id,
                            address,
                            error: P2pNetworkKademliaError::Timeout,
                            stream_id,
                        });
                    }
                }
            }
            Self::StreamSuccess {
                id,
                stream_id,
                peer_id,
                address,
            } => {
                let self_peer_id = store.state().config.identity_pub_key.peer_id();

                let message = find_node_request(self_peer_id);
                let data = Data(message.to_bytes().into());

                store.dispatch(Self::SendMessageInit {
                    id,
                    stream_id,
                    peer_id,
                    address,
                    data,
                });
            }
            Self::SendMessageInit {
                id,
                stream_id,
                peer_id,
                address,
                data,
            } => {
                if store.dispatch(P2pNetworkYamuxOutgoingDataAction {
                    addr: address,
                    stream_id,
                    data,
                    fin: false,
                }) {
                    store.dispatch(Self::SendMessagePending {
                        id,
                        stream_id,
                        peer_id,
                        address,
                        time: meta.time(),
                    });
                }
            }
            Self::SendMessagePending {
                time,
                id,
                stream_id,
                peer_id,
                address,
            } => {
                if let Some(time) = meta.time().checked_sub(time) {
                    if &time > config.timeout() {
                        store.dispatch(Self::SendMessageFailed {
                            id,
                            peer_id,
                            address,
                            error: P2pNetworkKademliaError::Timeout,
                            stream_id,
                        });
                    }
                }
            }
            Self::SendMessageSuccess { .. } => {}
            Self::ConnectFailed { .. } => {}
            Self::StreamFailed { .. } => {}
            _ => {
                dbg!(&self);
                todo!()
            }
        }
    }
}
