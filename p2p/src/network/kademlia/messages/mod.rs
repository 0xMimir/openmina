mod dht;

mod mappings;
pub use mappings::*;

use crate::{Data, P2pNetworkKademliaError, PeerId};

pub fn find_node_request(peer_id: PeerId) -> dht::Message {
    let peer_id = libp2p_identity::PeerId::from(peer_id);

    dht::Message {
        type_pb: dht::mod_Message::MessageType::FIND_NODE,
        clusterLevelRaw: 10,
        key: peer_id.to_bytes(),
        ..Default::default()
    }
}

pub fn find_node_response(closer_peers: Vec<Peer>) -> dht::Message {
    dht::Message {
        type_pb: dht::mod_Message::MessageType::FIND_NODE,
        clusterLevelRaw: 10,
        closerPeers: closer_peers.into_iter().map(From::from).collect(),
        ..Default::default()
    }
}

impl dht::Message {
    pub fn to_bytes(&self) -> Vec<u8> {
        quick_protobuf::serialize_into_vec(self).expect("Error serializing")
    }
}

pub fn bytes_to_message(data: &Data) -> Result<Message, P2pNetworkKademliaError> {
    let raw_message = quick_protobuf::deserialize_from_slice::<dht::Message>(data.0.as_ref())
        .map_err(|e| P2pNetworkKademliaError::Parse(e.to_string()))?;

    raw_message.try_into()
}
