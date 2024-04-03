use std::net::SocketAddr;

use redux::EnablingCondition;
use serde::{Deserialize, Serialize};

use crate::{
    kademlia::p2p_network_kademlia_state::P2pNetworkKademliaCurrentState,
    outgoing::P2pNetworkKademliaOutgoingAction, Data, P2pAction, P2pNetworkAction, P2pState,
    PeerId, StreamId,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum P2pNetworkKademliaAction {
    Bootstrap {
        initial_peers: Vec<(SocketAddr, PeerId)>,
        peer_id: PeerId,
    },
    BootstrapPending,
    BootstrapSuccess,
    Outgoing(P2pNetworkKademliaOutgoingAction),
    IncomingData {
        peer_id: PeerId,
        address: SocketAddr,
        stream_id: StreamId,
        data: Data,
    },
}

impl EnablingCondition<P2pState> for P2pNetworkKademliaAction {
    fn is_enabled(&self, state: &P2pState, time: redux::Timestamp) -> bool {
        let kademlia_state = &state.network.scheduler.discovery_state;

        match self {
            Self::Bootstrap { .. } => {
                matches!(
                    kademlia_state.current,
                    P2pNetworkKademliaCurrentState::BootstrapFailed { .. }
                        | P2pNetworkKademliaCurrentState::Uninitialized
                )
            }
            Self::BootstrapPending => matches!(
                kademlia_state.current,
                P2pNetworkKademliaCurrentState::BootstrapPending { .. }
            ),
            Self::BootstrapSuccess => matches!(
                kademlia_state.current,
                P2pNetworkKademliaCurrentState::BootstrapPending { .. }
            ),
            Self::Outgoing(a) => {
                let P2pNetworkKademliaCurrentState::BootstrapPending { .. } =
                    &kademlia_state.current
                else {
                    return false;
                };

                a.is_enabled(state, time)
            }
            Self::IncomingData { .. } => matches!(
                kademlia_state.current,
                P2pNetworkKademliaCurrentState::BootstrapPending { .. }
                    | P2pNetworkKademliaCurrentState::BootstrapSuccess { .. }
            ),
        }
    }
}

impl From<P2pNetworkKademliaAction> for P2pAction {
    fn from(value: P2pNetworkKademliaAction) -> Self {
        Self::Network(P2pNetworkAction::Kademlia(value))
    }
}
