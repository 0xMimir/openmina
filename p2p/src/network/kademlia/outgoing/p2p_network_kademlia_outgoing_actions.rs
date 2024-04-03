use std::net::SocketAddr;

use redux::EnablingCondition;
use serde::{Deserialize, Serialize};

use crate::{
    messages::Message, Data, P2pAction, P2pNetworkKademliaAction, P2pNetworkKademliaError, P2pState, PeerId, StreamId
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum P2pNetworkKademliaOutgoingAction {
    ConnectInit {
        id: usize,
        peer_id: PeerId,
        address: SocketAddr,
    },
    ConnectPending {
        id: usize,
        peer_id: PeerId,
        address: SocketAddr,
        time: redux::Timestamp
    },
    ConnectFailed {
        id: usize,
        peer_id: PeerId,
        address: SocketAddr,
        error: P2pNetworkKademliaError,
    },
    ConnectSuccess {
        id: usize,
        stream_id: StreamId,
        peer_id: PeerId,
        address: SocketAddr,
    },
    StreamInit {
        id: usize,
        stream_id: StreamId,
        peer_id: PeerId,
        address: SocketAddr,
    },
    StreamPending {
        id: usize,
        stream_id: StreamId,
        peer_id: PeerId,
        address: SocketAddr,
        time: redux::Timestamp
    },
    StreamFailed {
        id: usize,
        stream_id: StreamId,
        peer_id: PeerId,
        address: SocketAddr,
        error: P2pNetworkKademliaError,
    },
    StreamSuccess {
        id: usize,
        stream_id: StreamId,
        peer_id: PeerId,
        address: SocketAddr,
    },
    SendMessageInit {
        id: usize,
        stream_id: StreamId,
        peer_id: PeerId,
        address: SocketAddr,
        data: Data,
    },
    SendMessagePending {
        id: usize,
        stream_id: StreamId,
        peer_id: PeerId,
        address: SocketAddr,
        time: redux::Timestamp
    },
    SendMessageFailed {
        id: usize,
        stream_id: StreamId,
        peer_id: PeerId,
        address: SocketAddr,
        error: P2pNetworkKademliaError,
    },
    SendMessageSuccess {
        id: usize,
        stream_id: StreamId,
        peer_id: PeerId,
        address: SocketAddr,
        message: Message,
    },
}

impl EnablingCondition<P2pState> for P2pNetworkKademliaOutgoingAction {}

impl From<P2pNetworkKademliaOutgoingAction> for P2pNetworkKademliaAction {
    fn from(value: P2pNetworkKademliaOutgoingAction) -> Self {
        Self::Outgoing(value)
    }
}

impl From<P2pNetworkKademliaOutgoingAction> for P2pAction {
    fn from(value: P2pNetworkKademliaOutgoingAction) -> Self {
        P2pNetworkKademliaAction::from(value).into()
    }
}

impl P2pNetworkKademliaOutgoingAction {
    pub fn id(&self) -> usize {
        *match self {
            P2pNetworkKademliaOutgoingAction::ConnectInit { id, .. } => id,
            P2pNetworkKademliaOutgoingAction::ConnectPending { id, .. } => id,
            P2pNetworkKademliaOutgoingAction::ConnectFailed { id, .. } => id,
            P2pNetworkKademliaOutgoingAction::ConnectSuccess { id, .. } => id,
            P2pNetworkKademliaOutgoingAction::StreamInit { id, .. } => id,
            P2pNetworkKademliaOutgoingAction::StreamPending { id, .. } => id,
            P2pNetworkKademliaOutgoingAction::StreamFailed { id, .. } => id,
            P2pNetworkKademliaOutgoingAction::StreamSuccess { id, .. } => id,
            P2pNetworkKademliaOutgoingAction::SendMessageInit { id, .. } => id,
            P2pNetworkKademliaOutgoingAction::SendMessagePending { id, .. } => id,
            P2pNetworkKademliaOutgoingAction::SendMessageFailed { id, .. } => id,
            P2pNetworkKademliaOutgoingAction::SendMessageSuccess { id, .. } => id,
        }
    }
}
