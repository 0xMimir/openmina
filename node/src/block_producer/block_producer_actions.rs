use std::cmp::Ordering;

use mina_p2p_messages::v2::{
    ConsensusBodyReferenceStableV1, LedgerProofProdStableV2, MinaBaseStagedLedgerHashStableV1,
    StagedLedgerDiffDiffStableV2,
};
use openmina_core::block::ArcBlockWithHash;
use serde::{Deserialize, Serialize};

use super::vrf_evaluator::BlockProducerVrfEvaluatorAction;
use super::{BlockProducerCurrentState, BlockProducerWonSlot, BlockProducerWonSlotDiscardReason};

pub type BlockProducerActionWithMeta = redux::ActionWithMeta<BlockProducerAction>;
pub type BlockProducerActionWithMetaRef<'a> = redux::ActionWithMeta<&'a BlockProducerAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum BlockProducerAction {
    VrfEvaluator(BlockProducerVrfEvaluatorAction),
    BestTipUpdate(BlockProducerBestTipUpdateAction),
    WonSlotSearch(BlockProducerWonSlotSearchAction),
    WonSlot(BlockProducerWonSlotAction),
    WonSlotDiscard(BlockProducerWonSlotDiscardAction),
    WonSlotWait(BlockProducerWonSlotWaitAction),
    WonSlotProduceInit(BlockProducerWonSlotProduceInitAction),
    StagedLedgerDiffCreateInit(BlockProducerStagedLedgerDiffCreateInitAction),
    StagedLedgerDiffCreatePending(BlockProducerStagedLedgerDiffCreatePendingAction),
    StagedLedgerDiffCreateSuccess(BlockProducerStagedLedgerDiffCreateSuccessAction),
    BlockUnprovenBuild(BlockProducerBlockUnprovenBuildAction),
    BlockProduced(BlockProducerBlockProducedAction),
    BlockInject(BlockProducerBlockInjectAction),
    BlockInjected(BlockProducerBlockInjectedAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerBestTipUpdateAction {
    pub best_tip: ArcBlockWithHash,
}

impl redux::EnablingCondition<crate::State> for BlockProducerBestTipUpdateAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        true
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerWonSlotSearchAction {}

impl redux::EnablingCondition<crate::State> for BlockProducerWonSlotSearchAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .block_producer
            .with(false, |this| this.current.won_slot_should_search())
        // TODO(adonagy): check also if we have any won slots with higher
        // global slot than current best tip in transition frontier.
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerWonSlotAction {
    pub won_slot: BlockProducerWonSlot,
}

impl redux::EnablingCondition<crate::State> for BlockProducerWonSlotAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.block_producer.with(false, |this| {
            let Some(best_tip) = state.transition_frontier.best_tip() else {
                return false;
            };
            let is_won_slot_old = |won_slot: &BlockProducerWonSlot| {
                best_tip.global_slot() > won_slot.global_slot_since_genesis.as_u32()
            };

            state.time() < self.won_slot.next_slot_time()
                && is_won_slot_old(&self.won_slot)
                && this
                    .current
                    .won_slot()
                    .filter(|won_slot| !is_won_slot_old(won_slot))
                    .map_or(true, |won_slot| {
                        match self
                            .won_slot
                            .global_slot_since_genesis
                            .as_u32()
                            .cmp(&won_slot.global_slot_since_genesis.as_u32())
                        {
                            // old won_slot is further in the future, pick new one instead.
                            Ordering::Less => true,
                            Ordering::Equal => &self.won_slot > won_slot,
                            // new won_slot is further in the future. Ignore it.
                            Ordering::Greater => false,
                        }
                    })
            // TODO(binier): do we need to check if staking epoch ledger for an
            // existing won slot is still current best tip's staking epoch ledger.
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerWonSlotWaitAction {}

impl redux::EnablingCondition<crate::State> for BlockProducerWonSlotWaitAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.block_producer.with(false, |this| {
            this.current.won_slot_should_wait(state.time())
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerWonSlotProduceInitAction {}

impl redux::EnablingCondition<crate::State> for BlockProducerWonSlotProduceInitAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.block_producer.with(false, |this| {
            !this.current.won_slot_should_wait(state.time())
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerStagedLedgerDiffCreateInitAction {}

impl redux::EnablingCondition<crate::State> for BlockProducerStagedLedgerDiffCreateInitAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.block_producer.with(false, |this| {
            matches!(
                this.current,
                BlockProducerCurrentState::WonSlotProduceInit { .. }
            )
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerStagedLedgerDiffCreatePendingAction {}

impl redux::EnablingCondition<crate::State> for BlockProducerStagedLedgerDiffCreatePendingAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.block_producer.with(false, |this| {
            matches!(
                this.current,
                BlockProducerCurrentState::WonSlotProduceInit { .. }
            )
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerStagedLedgerDiffCreateSuccessAction {
    pub diff: StagedLedgerDiffDiffStableV2,
    pub diff_hash: ConsensusBodyReferenceStableV1,
    pub staged_ledger_hash: MinaBaseStagedLedgerHashStableV1,
    pub emitted_ledger_proof: Option<LedgerProofProdStableV2>,
}

impl redux::EnablingCondition<crate::State> for BlockProducerStagedLedgerDiffCreateSuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.block_producer.with(false, |this| {
            matches!(
                this.current,
                BlockProducerCurrentState::StagedLedgerDiffCreatePending { .. }
            )
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerBlockUnprovenBuildAction {}

impl redux::EnablingCondition<crate::State> for BlockProducerBlockUnprovenBuildAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.block_producer.with(false, |this| {
            matches!(
                this.current,
                BlockProducerCurrentState::StagedLedgerDiffCreateSuccess { .. }
            )
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerBlockProducedAction {}

impl redux::EnablingCondition<crate::State> for BlockProducerBlockProducedAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.block_producer.with(false, |this| {
            matches!(
                this.current,
                BlockProducerCurrentState::BlockUnprovenBuilt { .. }
            )
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerBlockInjectAction {}

impl redux::EnablingCondition<crate::State> for BlockProducerBlockInjectAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.block_producer.with(false, |this| {
            matches!(
                this.current,
                BlockProducerCurrentState::BlockUnprovenBuilt { .. }
            )
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerBlockInjectedAction {}

impl redux::EnablingCondition<crate::State> for BlockProducerBlockInjectedAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.block_producer.with(false, |this| {
            matches!(
                this.current,
                BlockProducerCurrentState::BlockUnprovenBuilt { .. }
            )
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerWonSlotDiscardAction {
    pub reason: BlockProducerWonSlotDiscardReason,
}

impl redux::EnablingCondition<crate::State> for BlockProducerWonSlotDiscardAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        todo!("validate reason")
    }
}

macro_rules! impl_into_global_action {
    ($a:ty) => {
        impl From<$a> for crate::Action {
            fn from(value: $a) -> Self {
                Self::BlockProducer(value.into())
            }
        }
    };
}

impl_into_global_action!(BlockProducerBestTipUpdateAction);
impl_into_global_action!(BlockProducerWonSlotSearchAction);
impl_into_global_action!(BlockProducerWonSlotAction);
impl_into_global_action!(BlockProducerWonSlotDiscardAction);
impl_into_global_action!(BlockProducerWonSlotWaitAction);
impl_into_global_action!(BlockProducerWonSlotProduceInitAction);
impl_into_global_action!(BlockProducerStagedLedgerDiffCreateInitAction);
impl_into_global_action!(BlockProducerStagedLedgerDiffCreatePendingAction);
impl_into_global_action!(BlockProducerStagedLedgerDiffCreateSuccessAction);
impl_into_global_action!(BlockProducerBlockUnprovenBuildAction);
impl_into_global_action!(BlockProducerBlockProducedAction);
impl_into_global_action!(BlockProducerBlockInjectAction);
impl_into_global_action!(BlockProducerBlockInjectedAction);