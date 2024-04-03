use redux::ActionWithMeta;
use std::{borrow::BorrowMut, collections::HashMap};

use super::{
    outgoing::P2pNetworkKademliaOutgoingState,
    p2p_network_kademlia_state::{P2pNetworkKademliaCurrentState, P2pNetworkKademliaRoutingTable},
    P2pNetworkKademliaAction, P2pNetworkKademliaState,
};

impl P2pNetworkKademliaState {
    pub fn reducer(&mut self, action: ActionWithMeta<&P2pNetworkKademliaAction>) {
        let (action, meta) = action.split();
        match action {
            P2pNetworkKademliaAction::Bootstrap {
                initial_peers,
                peer_id,
            } => {
                let routing_table = P2pNetworkKademliaRoutingTable::new(*peer_id, None);
                let mut pending_requests = initial_peers.iter().map(|(address, peer)| {
                    P2pNetworkKademliaOutgoingState::ConnectInit {
                        peer_id: *peer,
                        address: *address,
                    }
                });

                let mut active_requests = HashMap::with_capacity(self.config.alpha());
                let mut id = 0;
                while active_requests.len() < self.config.alpha() {
                    let Some(request) = pending_requests.next() else {
                        break;
                    };

                    active_requests.insert(id, request);
                    id += 1;
                }
                let pending_requests = pending_requests.collect::<Vec<_>>();

                self.current = P2pNetworkKademliaCurrentState::BootstrapPending {
                    active_requests,
                    pending_requests,
                    self_peer_id: *peer_id,
                    routing_table,
                }
            }
            P2pNetworkKademliaAction::BootstrapPending => {
                let P2pNetworkKademliaCurrentState::BootstrapPending {
                    active_requests,
                    routing_table,
                    pending_requests,
                    ..
                } = self.current.borrow_mut()
                else {
                    return;
                };

                let mut finished_requests = vec![];
                for (id, request) in active_requests.iter_mut() {
                    let id = match request {
                        P2pNetworkKademliaOutgoingState::SendMessageSuccess { message, .. } => {
                            for peer in &message.closer_peers {
                                routing_table.insert(peer.clone());
                                pending_requests.extend(peer.generate_new_requests());
                            }

                            id
                        }
                        P2pNetworkKademliaOutgoingState::ConnectFailed { .. } => id,
                        P2pNetworkKademliaOutgoingState::StreamFailed { .. } => id,
                        P2pNetworkKademliaOutgoingState::SendMessageFailed { .. } => id,
                        _ => continue,
                    };

                    finished_requests.push(*id);
                }

                for id in finished_requests {
                    active_requests.remove(&id);
                }

                while active_requests.len() < self.config.alpha() {
                    let Some(request) = pending_requests.pop() else {
                        break;
                    };
                    active_requests.insert(
                        active_requests.keys().max().map_or(0, |id| id + 1),
                        request.to_owned(),
                    );
                }
            }
            P2pNetworkKademliaAction::BootstrapSuccess => {
                let P2pNetworkKademliaCurrentState::BootstrapPending { routing_table, .. } =
                    self.current.borrow_mut()
                else {
                    return;
                };

                self.current = P2pNetworkKademliaCurrentState::BootstrapSuccess {
                    routing_table: routing_table.clone(),
                };
            }
            P2pNetworkKademliaAction::Outgoing(action) => {
                let P2pNetworkKademliaCurrentState::BootstrapPending {
                    active_requests, ..
                } = self.current.borrow_mut()
                else {
                    return;
                };

                let id = action.id();
                let Some(request_state) = active_requests.get_mut(&id) else {
                    return;
                };

                request_state.reducer(meta.with_action(action));
            }
            P2pNetworkKademliaAction::IncomingData { .. } => {}
        }
    }
}
