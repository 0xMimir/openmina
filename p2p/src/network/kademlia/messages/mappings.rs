use std::net::{IpAddr, SocketAddr};

use crate::{
    connection::outgoing::P2pConnectionOutgoingInitLibp2pOpts,
    outgoing::P2pNetworkKademliaOutgoingState, webrtc::Host, P2pNetworkKademliaError, PeerId,
};
use multiaddr::Multiaddr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message {
    pub type_pb: MessageType,
    pub cluster_level: i32,
    pub key: Vec<u8>,
    pub closer_peers: Vec<Peer>,
    pub provider_peers: Vec<Peer>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Peer {
    pub id: PeerId,
    pub addresses: Vec<Multiaddr>,
    pub connection: ConnectionType,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum MessageType {
    PutValue,
    GetValue,
    AddProvider,
    GetProviders,
    FindNode,
    Ping,
}

#[derive(Debug, Clone, Deserialize, Serialize, Copy)]
pub enum ConnectionType {
    NotConnected,
    Connected,
    CanConnect,
    CannotConnect,
}

impl From<super::dht::mod_Message::MessageType> for MessageType {
    fn from(value: super::dht::mod_Message::MessageType) -> Self {
        match value {
            super::dht::mod_Message::MessageType::PUT_VALUE => Self::PutValue,
            super::dht::mod_Message::MessageType::GET_VALUE => Self::GetValue,
            super::dht::mod_Message::MessageType::ADD_PROVIDER => Self::AddProvider,
            super::dht::mod_Message::MessageType::GET_PROVIDERS => Self::GetProviders,
            super::dht::mod_Message::MessageType::FIND_NODE => Self::FindNode,
            super::dht::mod_Message::MessageType::PING => Self::Ping,
        }
    }
}

impl From<super::dht::mod_Message::ConnectionType> for ConnectionType {
    fn from(value: super::dht::mod_Message::ConnectionType) -> Self {
        match value {
            super::dht::mod_Message::ConnectionType::NOT_CONNECTED => Self::NotConnected,
            super::dht::mod_Message::ConnectionType::CONNECTED => Self::Connected,
            super::dht::mod_Message::ConnectionType::CAN_CONNECT => Self::CanConnect,
            super::dht::mod_Message::ConnectionType::CANNOT_CONNECT => Self::CannotConnect,
        }
    }
}

impl From<ConnectionType> for super::dht::mod_Message::ConnectionType {
    fn from(value: ConnectionType) -> Self {
        match value {
            ConnectionType::CanConnect => Self::CAN_CONNECT,
            ConnectionType::CannotConnect => Self::CANNOT_CONNECT,
            ConnectionType::Connected => Self::CONNECTED,
            ConnectionType::NotConnected => Self::NOT_CONNECTED,
        }
    }
}

impl TryFrom<super::dht::Message> for Message {
    type Error = P2pNetworkKademliaError;

    fn try_from(value: super::dht::Message) -> Result<Self, Self::Error> {
        let super::dht::Message {
            type_pb,
            clusterLevelRaw,
            key,
            closerPeers,
            providerPeers,
            ..
        } = value;

        let mut closer_peers = vec![];
        for peer in closerPeers {
            closer_peers.push(Peer::try_from(peer)?);
        }

        let mut provider_peers = vec![];
        for peer in providerPeers {
            provider_peers.push(Peer::try_from(peer)?);
        }

        Ok(Self {
            type_pb: type_pb.into(),
            cluster_level: clusterLevelRaw,
            key,
            closer_peers,
            provider_peers,
        })
    }
}

impl TryFrom<super::dht::mod_Message::Peer> for Peer {
    type Error = P2pNetworkKademliaError;

    fn try_from(value: super::dht::mod_Message::Peer) -> Result<Self, Self::Error> {
        let super::dht::mod_Message::Peer {
            id,
            addrs,
            connection,
        } = value;

        let id = libp2p_identity::PeerId::from_bytes(&id)
            .map_err(|e| P2pNetworkKademliaError::Parse(e.to_string()))?
            .into();

        let mut addresses = vec![];
        for address in addrs {
            let multiaddr = Multiaddr::try_from(address)
                .map_err(|e| P2pNetworkKademliaError::Parse(e.to_string()))?;
            addresses.push(multiaddr);
        }

        Ok(Self {
            id,
            addresses,
            connection: connection.into(),
        })
    }
}

impl From<Peer> for super::dht::mod_Message::Peer {
    fn from(value: Peer) -> Self {
        Self {
            id: libp2p_identity::PeerId::from(value.id).to_bytes(),
            addrs: value
                .addresses
                .into_iter()
                .map(|address| address.to_vec())
                .collect(),
            connection: value.connection.into(),
        }
    }
}

impl Peer {
    pub fn generate_new_requests(&self) -> Vec<P2pNetworkKademliaOutgoingState> {
        let libp2p_peer_id = libp2p_identity::PeerId::from(self.id);

        self.addresses
            .iter()
            .filter_map(|multiaddr| {
                let multiaddr = multiaddr.clone().with_p2p(libp2p_peer_id).ok()?;
                let opts = P2pConnectionOutgoingInitLibp2pOpts::try_from(&multiaddr).ok()?;
                let host = match opts.host {
                    Host::Ipv4(ip) => {
                        if ip.is_loopback() || ip.is_private() {
                            None
                        } else {
                            Some(SocketAddr::new(IpAddr::V4(ip), opts.port))
                        }
                    }
                    Host::Ipv6(ip) => {
                        if ip.is_loopback() {
                            None
                        } else {
                            Some(SocketAddr::new(IpAddr::V6(ip), opts.port))
                        }
                    }
                    _ => None,
                }?;

                Some(P2pNetworkKademliaOutgoingState::ConnectInit {
                    peer_id: self.id,
                    address: host,
                })
            })
            .collect()
    }
}
