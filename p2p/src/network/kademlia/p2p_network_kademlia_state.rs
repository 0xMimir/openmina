use std::{collections::HashMap, net::SocketAddr};

use crate::{
    messages::{ConnectionType, Peer},
    PeerId, StreamId,
};
use crypto_bigint::{Encoding, U256};
use multiaddr::Multiaddr;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{
    outgoing::P2pNetworkKademliaOutgoingState,
    p2p_network_kademlia_config::P2pNetworkKademliaConfig,
};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct P2pNetworkKademliaState {
    pub config: P2pNetworkKademliaConfig,
    pub current: P2pNetworkKademliaCurrentState,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum P2pNetworkKademliaCurrentState {
    #[default]
    Uninitialized,
    BootstrapPending {
        active_requests: HashMap<usize, P2pNetworkKademliaOutgoingState>,
        pending_requests: Vec<P2pNetworkKademliaOutgoingState>,
        self_peer_id: PeerId,
        routing_table: P2pNetworkKademliaRoutingTable,
    },
    BootstrapFailed {},
    BootstrapSuccess {
        routing_table: P2pNetworkKademliaRoutingTable,
    },
}

impl P2pNetworkKademliaState {
    pub fn find_request(
        &self,
        peer_id: &PeerId,
        address: &SocketAddr,
        stream_id: &StreamId,
    ) -> Option<usize> {
        let P2pNetworkKademliaCurrentState::BootstrapPending {
            ref active_requests,
            ..
        } = &self.current
        else {
            return None;
        };

        active_requests
            .iter()
            .find(|(_, request)| {
                request.stream_id() == Some(stream_id)
                    && request.address() == address
                    && request.peer_id() == peer_id
            })
            .map(|(index, _)| *index)
    }

    pub fn get_routing_table(&self) -> Option<&P2pNetworkKademliaRoutingTable> {
        match &self.current {
            P2pNetworkKademliaCurrentState::Uninitialized => None,
            P2pNetworkKademliaCurrentState::BootstrapPending { routing_table, .. } => {
                Some(routing_table)
            }
            P2pNetworkKademliaCurrentState::BootstrapFailed {} => None,
            P2pNetworkKademliaCurrentState::BootstrapSuccess { routing_table } => {
                Some(routing_table)
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// see how to make default implementation
pub struct P2pNetworkKademliaRoutingTable {
    pub self_key: U256,                         // Change to proper to new type
    pub buckets: Vec<P2pNetworkKademliaBucket>, // -||-
    pub bucket_size: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct P2pNetworkKademliaBucket {
    pub entries: Vec<P2pNetworkKademliaBucketEntry>, // see about using map with peer id as key
    pub size: usize,                                 // move this to const
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct P2pNetworkKademliaBucketEntry {
    pub peer_id: PeerId,
    pub key: U256,
    pub addresses: Vec<Multiaddr>,
    pub connection: ConnectionType,
}

impl P2pNetworkKademliaRoutingTable {
    pub fn new(self_peer_id: PeerId, bucket_size: Option<usize>) -> Self {
        let bucket_size = bucket_size.unwrap_or(20);
        let self_key = U256::from_be_bytes(
            Sha256::digest(self_peer_id.to_bytes().to_vec())
                .try_into()
                .unwrap(),
        );

        Self {
            self_key,
            buckets: vec![
                // first entry should be itself, add later
                P2pNetworkKademliaBucket {
                    entries: vec![],
                    size: bucket_size,
                },
            ],
            bucket_size,
        }
    }

    pub fn insert(&mut self, peer: Peer) {
        let entry = P2pNetworkKademliaBucketEntry::new(peer);

        let distance = self.self_key ^ entry.key;
        let bucket_index = 256 - distance.bits_vartime();
        let max_index = self.buckets.len() - 1;

        if bucket_index < max_index {
            if self.buckets[bucket_index].entries.len() < self.buckets[bucket_index].size {
                self.buckets[bucket_index].insert(entry);
            }
        } else {
            if self.buckets[max_index].entries.len() < self.buckets[max_index].size {
                self.buckets[max_index].insert(entry);
            } else {
                let next_index = max_index + 1;
                let distance = U256::MAX >> next_index;

                let mut last_bucket = self
                    .buckets
                    .pop()
                    .expect("There must be at least one bucket");

                // this might result in more peers in bucket than allowed
                last_bucket.insert(entry);
                let (previous_bucket_entries, new_bucket_entries): (Vec<_>, Vec<_>) = last_bucket
                    .entries
                    .into_iter()
                    .partition(|entry| (entry.key ^ self.self_key) <= distance);

                self.buckets.extend([
                    P2pNetworkKademliaBucket {
                        entries: previous_bucket_entries,
                        size: self.bucket_size,
                    },
                    P2pNetworkKademliaBucket {
                        entries: new_bucket_entries,
                        size: self.bucket_size,
                    },
                ]);
            }
        }
    }

    pub fn find_closest_nodes(&self, peer_id: PeerId, k_param: usize) -> Vec<Peer> {
        let peer_id = U256::from_be_bytes(
            Sha256::digest(peer_id.to_bytes().to_vec())
                .try_into()
                .unwrap(),
        );
        let distance = self.self_key ^ peer_id;
        let bucket_index = 256 - distance.bits_vartime();
        let mut max_index = self.buckets.len() - 1;

        let mut nodes = vec![];

        // if corresponding bucket exists start with that bucket and go up and down from there
        if bucket_index < max_index {
            nodes.extend(self.buckets[bucket_index].entries.iter());
            let mut bucket_index_up = bucket_index + 1;
            let mut bucket_index_down = bucket_index - 1;

            while nodes.len() < k_param && (bucket_index_up < max_index || bucket_index_down > 0) {
                if bucket_index_down > 0 {
                    nodes.extend(self.buckets[bucket_index_down].entries.iter());
                    bucket_index_down -= 1;
                }

                if bucket_index_up < max_index {
                    nodes.extend(self.buckets[bucket_index_up].entries.iter());
                    bucket_index_up += 1;
                }
            }
        } else {
            // if there is only one bucket
            if max_index == 0 {
                nodes.extend(self.buckets[max_index].entries.iter());
            } else {
                // go from max bucket down
                while nodes.len() < k_param && max_index > 0 {
                    nodes.extend(self.buckets[max_index].entries.iter());
                    max_index -= 1;
                }
            }
        }

        // add distance from requested peer, sort by it and keep only closest
        let mut nodes = nodes
            .into_iter()
            .map(|peer| {
                let distance = peer.key ^ peer_id;

                (Peer::from(peer), distance)
            })
            .collect::<Vec<_>>();

        nodes.sort_by(|a, b| a.1.cmp(&b.1));
        nodes.truncate(k_param);
        nodes.into_iter().map(|(peer, _)| peer).collect()
    }
}

impl P2pNetworkKademliaBucket {
    pub fn insert(&mut self, peer: P2pNetworkKademliaBucketEntry) {
        self.entries.push(peer);
    }
}

impl P2pNetworkKademliaBucketEntry {
    fn new(value: Peer) -> Self {
        let Peer {
            id,
            addresses,
            connection,
        } = value;

        let key = U256::from_be_bytes(Sha256::digest(id.to_bytes().to_vec()).try_into().unwrap());

        Self {
            peer_id: id,
            addresses,
            connection,
            key,
        }
    }
}

impl From<&P2pNetworkKademliaBucketEntry> for Peer {
    fn from(value: &P2pNetworkKademliaBucketEntry) -> Self {
        let P2pNetworkKademliaBucketEntry {
            peer_id,
            addresses,
            connection,
            ..
        } = value;

        Self {
            id: *peer_id,
            addresses: addresses.clone(),
            connection: *connection,
        }
    }
}
