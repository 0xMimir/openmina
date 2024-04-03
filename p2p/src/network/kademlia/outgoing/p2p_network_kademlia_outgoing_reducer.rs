use redux::ActionWithMeta;

use super::{
    p2p_network_kademlia_outgoing_actions::P2pNetworkKademliaOutgoingAction,
    P2pNetworkKademliaOutgoingState,
};

impl P2pNetworkKademliaOutgoingState {
    pub fn reducer(&mut self, action: ActionWithMeta<&P2pNetworkKademliaOutgoingAction>) {
        let (action, _meta) = action.split();

        match action {
            P2pNetworkKademliaOutgoingAction::ConnectInit {
                peer_id, address, ..
            } => {
                *self = Self::ConnectInit {
                    peer_id: *peer_id,
                    address: *address,
                }
            }
            P2pNetworkKademliaOutgoingAction::ConnectPending {
                peer_id,
                address,
                time,
                ..
            } => {
                *self = Self::ConnectPending {
                    peer_id: *peer_id,
                    address: *address,
                    time: *time,
                }
            }
            P2pNetworkKademliaOutgoingAction::ConnectSuccess {
                stream_id,
                peer_id,
                address,
                ..
            } => {
                *self = Self::ConnectSuccess {
                    peer_id: *peer_id,
                    address: *address,
                    stream_id: *stream_id,
                };
            }
            P2pNetworkKademliaOutgoingAction::StreamInit {
                stream_id,
                peer_id,
                address,
                ..
            } => {
                *self = Self::StreamInit {
                    stream_id: *stream_id,
                    peer_id: *peer_id,
                    address: *address,
                }
            }
            P2pNetworkKademliaOutgoingAction::StreamPending {
                stream_id,
                peer_id,
                address,
                time,
                ..
            } => {
                *self = Self::StreamPending {
                    stream_id: *stream_id,
                    peer_id: *peer_id,
                    address: *address,
                    time: *time,
                }
            }
            P2pNetworkKademliaOutgoingAction::StreamSuccess {
                stream_id,
                peer_id,
                address,
                ..
            } => {
                *self = Self::StreamSuccess {
                    stream_id: *stream_id,
                    peer_id: *peer_id,
                    address: *address,
                }
            }
            P2pNetworkKademliaOutgoingAction::SendMessageInit {
                stream_id,
                peer_id,
                address,
                data,
                ..
            } => {
                *self = Self::SendMessageInit {
                    stream_id: *stream_id,
                    peer_id: *peer_id,
                    address: *address,
                    data: data.clone(),
                };
            }
            P2pNetworkKademliaOutgoingAction::SendMessagePending {
                stream_id,
                peer_id,
                address,
                time,
                ..
            } => {
                *self = Self::SendMessagePending {
                    stream_id: *stream_id,
                    peer_id: *peer_id,
                    address: *address,
                    time: *time,
                };
            }
            P2pNetworkKademliaOutgoingAction::SendMessageSuccess {
                stream_id,
                peer_id,
                address,
                message,
                ..
            } => {
                *self = Self::SendMessageSuccess {
                    stream_id: *stream_id,
                    peer_id: *peer_id,
                    address: *address,
                    message: message.clone(),
                }
            }
            P2pNetworkKademliaOutgoingAction::ConnectFailed {
                peer_id,
                address,
                error,
                ..
            } => {
                *self = Self::ConnectFailed {
                    peer_id: *peer_id,
                    address: *address,
                    error: error.clone(),
                }
            }
            P2pNetworkKademliaOutgoingAction::StreamFailed {
                stream_id,
                peer_id,
                address,
                error,
                ..
            } => {
                *self = Self::StreamFailed {
                    peer_id: *peer_id,
                    address: *address,
                    stream_id: *stream_id,
                    error: error.clone(),
                }
            }
            P2pNetworkKademliaOutgoingAction::SendMessageFailed {
                stream_id,
                peer_id,
                address,
                error,
                ..
            } => {
                *self = Self::SendMessageFailed {
                    stream_id: *stream_id,
                    peer_id: *peer_id,
                    address: *address,
                    error: error.clone(),
                }
            }
        }
    }
}
