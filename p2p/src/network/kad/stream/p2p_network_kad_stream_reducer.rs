use quick_protobuf::{serialize_into_vec, BytesReader};
use redux::ActionWithMeta;

use crate::{P2pNetworkKademliaRpcReply, P2pNetworkKademliaRpcRequest};

use super::{
    super::Message, P2pNetworkKadIncomingStreamState, P2pNetworkKadOutgoingStreamState,
    P2pNetworkKadStreamState, P2pNetworkKademliaStreamAction,
};

impl P2pNetworkKadIncomingStreamState {
    pub fn reducer(
        &mut self,
        action: ActionWithMeta<&P2pNetworkKademliaStreamAction>,
    ) -> Result<(), String> {
        use super::P2pNetworkKadIncomingStreamState as S;
        use super::P2pNetworkKademliaStreamAction as A;

        let (action, _meta) = action.split();

        match (&self, action) {
            (S::Default, A::New { incoming, .. }) if *incoming == true => {
                *self = S::WaitingForRequest {
                    expect_close: false,
                };
                Ok(())
            }
            (S::WaitingForRequest { .. }, A::IncomingData { data, .. }) => {
                let data = &data.0;

                let mut reader = BytesReader::from_bytes(data);
                let Ok(len) = reader.read_varint32(data).map(|v| v as usize) else {
                    *self = S::Error("error reading message length".to_owned());
                    return Ok(());
                };

                if len > reader.len() {
                    *self = S::PartialRequestReceived {
                        len,
                        data: data[(len - reader.len())..].to_vec(),
                    };
                    return Ok(());
                }

                self.handle_incoming_request(len, &data[data.len() - reader.len()..])
            }
            (S::PartialRequestReceived { len, data }, A::IncomingData { data: new_data, .. }) => {
                let mut data = data.clone();
                data.extend_from_slice(&new_data.0);

                if *len > data.len() {
                    *self = S::PartialRequestReceived { len: *len, data };
                    return Ok(());
                }

                self.handle_incoming_request(*len, &data)
            }
            (S::RequestIsReady { .. }, A::WaitOutgoing { .. }) => {
                *self = S::WaitingForReply;
                Ok(())
            }
            (S::WaitingForReply, A::SendResponse { data, .. }) => {
                let message = Message::from(data);
                let bytes = serialize_into_vec(&message).map_err(|e| format!("{e}"))?;
                *self = S::ResponseBytesAreReady { bytes };
                Ok(())
            }
            (S::ResponseBytesAreReady { .. }, A::WaitIncoming { .. }) => {
                *self = S::WaitingForRequest { expect_close: true };
                Ok(())
            }
            (S::WaitingForRequest { expect_close, .. }, A::RemoteClose { .. }) if *expect_close => {
                *self = S::Closing;
                Ok(())
            }
            _ => {
                return Err(format!(
                    "kademlia incoming stream state {self:?} is incorrect for action {action:?}",
                ));
            }
        }
    }

    fn handle_incoming_request(&mut self, len: usize, data: &[u8]) -> Result<(), String> {
        use super::P2pNetworkKadIncomingStreamState::*;

        let mut reader = BytesReader::from_bytes(data);

        let message = match reader.read_message_by_len::<Message>(data, len) {
            Ok(v) => v,
            Err(e) => {
                *self = Error(format!("error reading protobuf message: {e}"));
                return Ok(());
            }
        };

        let data = match P2pNetworkKademliaRpcRequest::try_from(message.clone()) {
            Ok(v) => v,
            Err(e) => {
                *self = Error(format!("error converting protobuf message: {e}"));
                return Ok(());
            }
        };

        *self = P2pNetworkKadIncomingStreamState::RequestIsReady { data };
        Ok(())
    }
}

impl P2pNetworkKadOutgoingStreamState {
    pub fn reducer(
        &mut self,
        action: ActionWithMeta<&P2pNetworkKademliaStreamAction>,
    ) -> Result<(), String> {
        use super::P2pNetworkKadOutgoingStreamState as S;
        use super::P2pNetworkKademliaStreamAction as A;
        let (action, _meta) = action.split();
        // println!("=== state:  {self:?}");
        // println!("=== action: {action:?}");
        match (&self, action) {
            (S::Default, A::New { incoming, .. }) if !*incoming => {
                *self = S::WaitingForRequest {
                    expect_close: false,
                };
                Ok(())
            }

            (S::WaitingForRequest { .. }, A::SendRequest { data, .. }) => {
                let message = Message::from(data);
                let bytes = serialize_into_vec(&message).map_err(|e| format!("{e}"))?;
                *self = S::RequestBytesAreReady { bytes };
                Ok(())
            }
            (S::RequestBytesAreReady { .. }, A::WaitIncoming { .. }) => {
                *self = S::WaitingForReply;
                Ok(())
            }

            (S::WaitingForReply { .. }, A::IncomingData { data, .. }) => {
                let data = &data.0;

                let mut reader = BytesReader::from_bytes(data);
                let Ok(len) = reader.read_varint32(data).map(|v| v as usize) else {
                    *self = S::Error("error reading message length".to_owned());
                    return Ok(());
                };

                if len > reader.len() {
                    *self = S::PartialReplyReceived {
                        len,
                        data: data[(len - reader.len())..].to_vec(),
                    };
                    return Ok(());
                }

                self.handle_incoming_response(len, &data[data.len() - reader.len()..])
            }
            (S::PartialReplyReceived { len, data }, A::IncomingData { data: new_data, .. }) => {
                let mut data = data.clone();
                data.extend_from_slice(&new_data.0);

                if *len > data.len() {
                    *self = S::PartialReplyReceived { len: *len, data };
                    return Ok(());
                }

                self.handle_incoming_response(*len, &data)
            }
            (S::ResponseIsReady { .. }, A::WaitOutgoing { .. }) => {
                *self = S::WaitingForRequest { expect_close: true };
                Ok(())
            }
            (S::WaitingForRequest { expect_close }, A::Close { .. }) if *expect_close => {
                *self = S::RequestBytesAreReady { bytes: Vec::new() };
                Ok(())
            }
            (S::Closing, A::RemoteClose { .. }) => {
                *self = S::Closed;
                Ok(())
            }
            _ => {
                return Err(format!(
                    "kademlia outgoing stream state {self:?} is incorrect for action {action:?}",
                ));
            }
        }
    }

    fn handle_incoming_response(&mut self, len: usize, data: &[u8]) -> Result<(), String> {
        use super::P2pNetworkKadOutgoingStreamState::*;

        let mut reader = BytesReader::from_bytes(data);

        let message = match reader.read_message_by_len::<Message>(data, len) {
            Ok(v) => v,
            Err(e) => {
                *self = Error(format!("error reading protobuf message: {e}"));
                return Ok(());
            }
        };

        let data = match P2pNetworkKademliaRpcReply::try_from(message.clone()) {
            Ok(v) => v,
            Err(e) => {
                *self = Error(format!("error converting protobuf message: {e}"));
                return Ok(());
            }
        };

        *self = P2pNetworkKadOutgoingStreamState::ResponseIsReady { data };
        Ok(())
    }
}

impl P2pNetworkKadStreamState {
    pub fn reducer(
        &mut self,
        action: ActionWithMeta<&P2pNetworkKademliaStreamAction>,
    ) -> Result<(), String> {
        match self {
            P2pNetworkKadStreamState::Incoming(i) => i.reducer(action),
            P2pNetworkKadStreamState::Outgoing(o) => o.reducer(action),
        }
    }
}