use serde::{Deserialize, Serialize};

use crate::p2p::channels::snark_job_commitment::SnarkJobCommitment;
use crate::p2p::channels::snark_job_commitment::SnarkJobId;
use crate::p2p::PeerId;

pub type JobCommitmentActionWithMeta = redux::ActionWithMeta<JobCommitmentAction>;
pub type JobCommitmentActionWithMetaRef<'a> = redux::ActionWithMeta<&'a JobCommitmentAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum JobCommitmentAction {
    Create(JobCommitmentCreateAction),
    Add(JobCommitmentAddAction),
    P2pSendAll(JobCommitmentP2pSendAllAction),
    P2pSend(JobCommitmentP2pSendAction),
    CheckTimeouts(JobCommitmentCheckTimeoutsAction),
    Timeout(JobCommitmentTimeoutAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JobCommitmentCreateAction {
    pub job_id: SnarkJobId,
}

impl redux::EnablingCondition<crate::State> for JobCommitmentCreateAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.job_commitments.should_create_commitment(&self.job_id)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JobCommitmentAddAction {
    pub commitment: SnarkJobCommitment,
    pub sender: PeerId,
}

impl redux::EnablingCondition<crate::State> for JobCommitmentAddAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        !state.job_commitments.contains(&self.commitment.job_id)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JobCommitmentP2pSendAllAction {}

impl redux::EnablingCondition<crate::State> for JobCommitmentP2pSendAllAction {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JobCommitmentP2pSendAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<crate::State> for JobCommitmentP2pSendAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.p2p.get_ready_peer(&self.peer_id).map_or(false, |p| {
            let (next_index, limit) = p.channels.snark_job_commitment.next_send_index_and_limit();
            limit > 0 && next_index <= state.job_commitments.last_index()
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JobCommitmentCheckTimeoutsAction {}

impl redux::EnablingCondition<crate::State> for JobCommitmentCheckTimeoutsAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .time()
            .checked_sub(state.job_commitments.last_check_timeouts)
            .map_or(false, |dur| dur.as_secs() >= 5)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JobCommitmentTimeoutAction {
    pub job_id: SnarkJobId,
}

impl redux::EnablingCondition<crate::State> for JobCommitmentTimeoutAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .job_commitments
            .is_commitment_timed_out(&self.job_id, state.time())
    }
}

macro_rules! impl_into_global_action {
    ($a:ty) => {
        impl From<$a> for crate::Action {
            fn from(value: $a) -> Self {
                Self::JobCommitment(value.into())
            }
        }
    };
}

impl_into_global_action!(JobCommitmentCreateAction);
impl_into_global_action!(JobCommitmentAddAction);
impl_into_global_action!(JobCommitmentP2pSendAllAction);
impl_into_global_action!(JobCommitmentP2pSendAction);
impl_into_global_action!(JobCommitmentCheckTimeoutsAction);
impl_into_global_action!(JobCommitmentTimeoutAction);