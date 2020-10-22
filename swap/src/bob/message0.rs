use libp2p::{
    request_response::{
        handler::RequestProtocol, ProtocolSupport, RequestResponse, RequestResponseConfig,
        RequestResponseEvent, RequestResponseMessage,
    },
    swarm::{NetworkBehaviourAction, NetworkBehaviourEventProcess, PollParameters},
    NetworkBehaviour, PeerId,
};
use std::{
    collections::VecDeque,
    task::{Context, Poll},
    time::Duration,
};
use tracing::error;

use crate::network::request_response::{AliceToBob, BobToAlice, Codec, Protocol};
use xmr_btc::{alice, bob};

#[derive(Debug)]
pub enum OutEvent {
    Msg(alice::Message0),
}

/// A `NetworkBehaviour` that represents send/recv of message 0.
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "OutEvent", poll_method = "poll")]
#[allow(missing_debug_implementations)]
pub struct Message0 {
    rr: RequestResponse<Codec>,
    #[behaviour(ignore)]
    events: VecDeque<OutEvent>,
}

impl Message0 {
    pub fn new(timeout: Duration) -> Self {
        let mut config = RequestResponseConfig::default();
        config.set_request_timeout(timeout);

        Self {
            rr: RequestResponse::new(
                Codec::default(),
                vec![(Protocol, ProtocolSupport::Full)],
                config,
            ),
            events: Default::default(),
        }
    }

    pub fn send(&mut self, alice: PeerId, msg: bob::Message0) {
        let msg = BobToAlice::Message0(msg);
        let _id = self.rr.send_request(&alice, msg);
    }

    fn poll(
        &mut self,
        _: &mut Context<'_>,
        _: &mut impl PollParameters,
    ) -> Poll<NetworkBehaviourAction<RequestProtocol<Codec>, OutEvent>> {
        if let Some(event) = self.events.pop_front() {
            return Poll::Ready(NetworkBehaviourAction::GenerateEvent(event));
        }

        Poll::Pending
    }
}

impl NetworkBehaviourEventProcess<RequestResponseEvent<BobToAlice, AliceToBob>> for Message0 {
    fn inject_event(&mut self, event: RequestResponseEvent<BobToAlice, AliceToBob>) {
        match event {
            RequestResponseEvent::Message {
                peer: _,
                message: RequestResponseMessage::Request { .. },
            } => panic!("Bob should never get a request from Alice"),
            RequestResponseEvent::Message {
                peer: _,
                message:
                    RequestResponseMessage::Response {
                        response,
                        request_id: _,
                    },
            } => match response {
                AliceToBob::Message0(msg) => self.events.push_back(OutEvent::Msg(msg)),
                other => panic!("unexpected response: {:?}", other),
            },

            RequestResponseEvent::InboundFailure { .. } => {
                panic!("Bob should never get a request from Alice, so should never get an InboundFailure");
            }
            RequestResponseEvent::OutboundFailure {
                peer: _,
                request_id: _,
                error,
            } => {
                error!("Outbound failure: {:?}", error);
            }
        }
    }
}
