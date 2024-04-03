mod p2p_network_kademlia_actions;
pub use p2p_network_kademlia_actions::P2pNetworkKademliaAction;

mod p2p_network_kademlia_config;

mod p2p_network_kademlia_effects;

mod p2p_network_kademlia_reducer;

mod p2p_network_kademlia_state;
pub use p2p_network_kademlia_state::{P2pNetworkKademliaCurrentState, P2pNetworkKademliaState};

pub mod messages;
pub mod outgoing;

#[derive(thiserror::Error, Debug, serde::Deserialize, serde::Serialize, Clone)]
pub enum P2pNetworkKademliaError {
    #[error("Timeout error")]
    Timeout,
    #[error("Error parsing failed: {0}")]
    Parse(String),
}
