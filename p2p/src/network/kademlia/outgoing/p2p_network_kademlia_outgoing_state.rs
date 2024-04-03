use crate::{messages::Message, Data, P2pNetworkKademliaError, PeerId, StreamId};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum P2pNetworkKademliaOutgoingState {
    ConnectInit {
        peer_id: PeerId,
        address: SocketAddr,
    },
    ConnectPending {
        peer_id: PeerId,
        address: SocketAddr,
        time: redux::Timestamp
    },
    ConnectFailed {
        peer_id: PeerId,
        address: SocketAddr,
        error: P2pNetworkKademliaError
    },
    ConnectSuccess {
        peer_id: PeerId,
        address: SocketAddr,
        stream_id: StreamId,
    },
    StreamInit {
        peer_id: PeerId,
        address: SocketAddr,
        stream_id: StreamId,
    },
    StreamPending {
        peer_id: PeerId,
        address: SocketAddr,
        stream_id: StreamId,
        time: redux::Timestamp
    },
    StreamFailed {
        peer_id: PeerId,
        address: SocketAddr,
        stream_id: StreamId,
        error: P2pNetworkKademliaError
    },
    StreamSuccess {
        peer_id: PeerId,
        address: SocketAddr,
        stream_id: StreamId,
    },
    SendMessageInit {
        stream_id: StreamId,
        peer_id: PeerId,
        address: SocketAddr,
        data: Data,
    },
    SendMessagePending {
        stream_id: StreamId,
        peer_id: PeerId,
        address: SocketAddr,
        time: redux::Timestamp
    },
    SendMessageFailed {
        stream_id: StreamId,
        peer_id: PeerId,
        address: SocketAddr,
        error: P2pNetworkKademliaError,
    },
    SendMessageSuccess {
        stream_id: StreamId,
        peer_id: PeerId,
        address: SocketAddr,
        message: Message,
    },
}

macro_rules! get_value {
    ($value:ident, $key:ident) => {
        match $value {
            P2pNetworkKademliaOutgoingState::ConnectInit { $key, .. } => $key,
            P2pNetworkKademliaOutgoingState::ConnectPending { $key, .. } => $key,
            P2pNetworkKademliaOutgoingState::ConnectFailed { $key, .. } => $key,
            P2pNetworkKademliaOutgoingState::ConnectSuccess { $key, .. } => $key,
            P2pNetworkKademliaOutgoingState::StreamInit { $key, .. } => $key,
            P2pNetworkKademliaOutgoingState::StreamPending { $key, .. } => $key,
            P2pNetworkKademliaOutgoingState::StreamFailed { $key, .. } => $key,
            P2pNetworkKademliaOutgoingState::StreamSuccess { $key, .. } => $key,
            P2pNetworkKademliaOutgoingState::SendMessageInit { $key, .. } => $key,
            P2pNetworkKademliaOutgoingState::SendMessagePending { $key, .. } => $key,
            P2pNetworkKademliaOutgoingState::SendMessageFailed { $key, .. } => $key,
            P2pNetworkKademliaOutgoingState::SendMessageSuccess { $key, .. } => $key,
        }
    };
}

impl P2pNetworkKademliaOutgoingState {
    pub fn peer_id(&self) -> &PeerId {
        get_value!(self, peer_id)
    }

    pub fn address(&self) -> &SocketAddr {
        get_value!(self, address)
    }

    pub fn stream_id(&self) -> Option<&StreamId> {
        match self {
            P2pNetworkKademliaOutgoingState::ConnectInit { .. } => None,
            P2pNetworkKademliaOutgoingState::ConnectPending { .. } => None,
            P2pNetworkKademliaOutgoingState::ConnectFailed { .. } => None,
            P2pNetworkKademliaOutgoingState::ConnectSuccess { .. } => None,
            P2pNetworkKademliaOutgoingState::StreamInit { stream_id, .. } => Some(stream_id),
            P2pNetworkKademliaOutgoingState::StreamPending { stream_id, .. } => Some(stream_id),
            P2pNetworkKademliaOutgoingState::StreamFailed { stream_id, .. } => Some(stream_id),
            P2pNetworkKademliaOutgoingState::StreamSuccess { stream_id, .. } => Some(stream_id),
            P2pNetworkKademliaOutgoingState::SendMessageInit { stream_id, .. } => Some(stream_id),
            P2pNetworkKademliaOutgoingState::SendMessagePending { stream_id, .. } => {
                Some(stream_id)
            }
            P2pNetworkKademliaOutgoingState::SendMessageFailed { stream_id, .. } => Some(stream_id),
            P2pNetworkKademliaOutgoingState::SendMessageSuccess { stream_id, .. } => {
                Some(stream_id)
            }
        }
    }
}
