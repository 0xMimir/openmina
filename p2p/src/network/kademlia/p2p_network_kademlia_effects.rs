use redux::ActionMeta;

use crate::{
    messages::{bytes_to_message, find_node_response, MessageType},
    outgoing::{P2pNetworkKademliaOutgoingAction, P2pNetworkKademliaOutgoingState},
    P2pNetworkYamuxOutgoingDataAction, P2pStore,
};

use super::{p2p_network_kademlia_state::P2pNetworkKademliaCurrentState, P2pNetworkKademliaAction};

impl P2pNetworkKademliaAction {
    pub fn effects<Store, S>(self, meta: &ActionMeta, store: &mut Store)
    where
        Store: P2pStore<S>,
    {
        let state = &store.state().network.scheduler.discovery_state;

        match self {
            Self::Bootstrap { .. } => {}
            Self::BootstrapPending => {
                let P2pNetworkKademliaCurrentState::BootstrapPending {
                    ref active_requests,
                    ref pending_requests,
                    ..
                } = &state.current
                else {
                    return;
                };

                let mut dispatch_actions = vec![];
                for (index, request) in active_requests.iter() {
                    let action = match request {
                        P2pNetworkKademliaOutgoingState::ConnectInit { peer_id, address } => {
                            P2pNetworkKademliaOutgoingAction::ConnectInit {
                                id: *index,
                                peer_id: *peer_id,
                                address: *address,
                            }
                        }
                        P2pNetworkKademliaOutgoingState::ConnectPending {
                            peer_id,
                            address,
                            time,
                        } => P2pNetworkKademliaOutgoingAction::ConnectPending {
                            id: *index,
                            peer_id: *peer_id,
                            address: *address,
                            time: *time,
                        },
                        P2pNetworkKademliaOutgoingState::StreamPending {
                            peer_id,
                            address,
                            stream_id,
                            time,
                        } => P2pNetworkKademliaOutgoingAction::StreamPending {
                            id: *index,
                            stream_id: *stream_id,
                            peer_id: *peer_id,
                            address: *address,
                            time: *time,
                        },
                        P2pNetworkKademliaOutgoingState::SendMessagePending {
                            stream_id,
                            peer_id,
                            address,time
                        } => P2pNetworkKademliaOutgoingAction::SendMessagePending {
                            id: *index,
                            stream_id: *stream_id,
                            peer_id: *peer_id,
                            address: *address,
                            time: *time,
                        },
                        _ => {
                            dbg!(&request);
                            todo!();
                        }
                    };

                    dispatch_actions.push(action);
                }

                if active_requests.is_empty() && pending_requests.is_empty(){
                    store.dispatch(Self::BootstrapSuccess);
                }

                for action in dispatch_actions {
                    store.dispatch(action);
                }
            }
            Self::BootstrapSuccess => {}
            Self::Outgoing(a) => {
                a.effects(meta, store);
            }
            Self::IncomingData {
                peer_id,
                stream_id,
                address,
                data,
            } => {
                let request_id = state.find_request(&peer_id, &address, &stream_id);
                let message = bytes_to_message(&data);

                match (request_id, message) {
                    (Some(id), Ok(message)) => {
                        store.dispatch(P2pNetworkKademliaOutgoingAction::SendMessageSuccess {
                            id,
                            stream_id,
                            peer_id,
                            address,
                            message,
                        });
                    }
                    (Some(id), Err(error)) => {
                        store.dispatch(P2pNetworkKademliaOutgoingAction::SendMessageFailed {
                            id,
                            stream_id,
                            peer_id,
                            address,
                            error,
                        });
                    }
                    (None, Ok(data)) => {
                        if data.type_pb != MessageType::FindNode {
                            return;
                        }

                        let Some(routing_table) = state.get_routing_table() else {
                            return;
                        };

                        let Ok(peer_id) = libp2p_identity::PeerId::from_bytes(&data.key) else {
                            return;
                        };

                        let closer_peers = routing_table
                            .find_closest_nodes(peer_id.into(), state.config.k_param());
                        let data = find_node_response(closer_peers).to_bytes().into();

                        store.dispatch(P2pNetworkYamuxOutgoingDataAction {
                            addr: address,
                            stream_id,
                            data,
                            fin: false,
                        });
                    }
                    (None, Err(_)) => todo!(),
                }
            }
        }
    }
}
