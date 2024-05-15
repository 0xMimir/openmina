use openmina_node_testing::scenarios::p2p::{kademlia::{IncomingFindNode, KademliaBootstrap}, kad::KadBootstrap};

mod common;

scenario_test!(incoming_find_node, IncomingFindNode, IncomingFindNode);

scenario_test!(kademlia_bootstrap, KademliaBootstrap, KademliaBootstrap);

scenario_test!(kad_bootstrap, KadBootstrap, KadBootstrap);