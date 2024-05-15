use crate::{
    cluster::ClusterNodeId,
    node::{DaemonJson, OcamlNodeTestingConfig, RustNodeTestingConfig},
    scenario::ListenerNode,
    scenarios::{ClusterRunner, Driver, RunCfg},
};
use node::{
    event_source::Event,
    p2p::{
        connection::outgoing::{
            P2pConnectionOutgoingInitLibp2pOpts, P2pConnectionOutgoingInitOpts,
        },
        identity::SecretKey,
        webrtc::Host,
    },
    ActionKind,
};
use std::time::Duration;

/// Kademlia Bootstrap test
/// 1. Create seed node
/// 2. Create node 1 with seed node as initial peer
/// 3. Wait for bootstrap with node 1
/// 4. Create node 2 with node 1 as initial peer
/// 5. Wait for bootstrap with node 2
/// 6. Check that node 2 knows of both seed node and node 1
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct KadBootstrap;

impl KadBootstrap {
    pub async fn run<'a>(&self, mut cluster: ClusterRunner<'a>) {
        std::env::set_var("OPENMINA_DISCOVERY_FILTER_ADDR", "false");

        let ocaml_node = cluster.add_ocaml_node(OcamlNodeTestingConfig {
            initial_peers: vec![],
            daemon_json: DaemonJson::Custom("/var/lib/coda/berkeley.json".to_owned()),
            block_producer: None,
        });

        let node1 = cluster.add_rust_node(
            RustNodeTestingConfig::berkeley_default()
                // .initial_peers(vec![ListenerNode::Ocaml(ocaml_node)]),
        );

        let node1_peer_id = cluster.node(node1).unwrap().peer_id();
        let node2 = cluster.add_rust_node(
            RustNodeTestingConfig::berkeley_default()
                .initial_peers(vec![ListenerNode::Rust(node1)]),
        );
        let node2_peer_id = cluster.node(node2).unwrap().peer_id();

        wait_for_kademlia_bootstrap(&mut cluster, node2)
            .await
            .expect("Kademlia not bootstrapped");

        let node3 = cluster.add_rust_node(
            RustNodeTestingConfig::berkeley_default()
                .initial_peers(vec![ListenerNode::Rust(node2)]),
        );
        let node3_peer_id = cluster.node(node3).unwrap().peer_id();

        wait_for_kademlia_bootstrap(&mut cluster, node3)
            .await
            .expect("Kademlia not bootstrapped");

        let bucket = cluster
            .node(node3)
            .unwrap()
            .state()
            .p2p
            .network
            .scheduler
            .discovery_state()
            .unwrap()
            .routing_table
            .buckets
            .first()
            .expect("Must have at least one bucket");

        let seed_peer = bucket.iter().find(|peer| peer.peer_id == node1_peer_id);
        assert!(seed_peer.is_some(), "Seed peer not found");

        let seed_peer = bucket.iter().find(|peer| peer.peer_id == node2_peer_id);
        assert!(seed_peer.is_some(), "Peer 1 peer not found");

        let self_peer = bucket.iter().find(|peer| peer.peer_id == node3_peer_id);
        assert!(self_peer.is_some(), "Self peer not found");
    }
}

async fn wait_for_kademlia_bootstrap<'a>(
    cluster: &mut ClusterRunner<'a>,
    node_id: ClusterNodeId,
) -> Result<(), anyhow::Error> {
    cluster
        .run(
            RunCfg::default()
                .timeout(Duration::from_secs(5))
                .action_handler(move |id, _, _, action| {
                    matches!(
                        (action.action().kind(), id),
                        (ActionKind::P2pNetworkKademliaBootstrapFinished, node_id)
                    )
                }),
        )
        .await
}
