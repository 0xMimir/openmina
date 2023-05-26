pub mod snark_job_commitment;
use snark_job_commitment::SnarkJobCommitmentPropagationChannelMsg;

mod p2p_channels_state;
pub use p2p_channels_state::*;

mod p2p_channels_actions;
pub use p2p_channels_actions::*;

mod p2p_channels_reducer;
pub use p2p_channels_reducer::*;

mod p2p_channels_effects;
pub use p2p_channels_effects::*;

mod p2p_channels_service;
pub use p2p_channels_service::*;

use binprot::{BinProtRead, BinProtWrite};
use binprot_derive::{BinProtRead, BinProtWrite};
use derive_more::From;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(Serialize, Deserialize, EnumIter, Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum ChannelId {
    SnarkJobCommitmentPropagation = 5,
}

impl ChannelId {
    #[inline(always)]
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    #[inline(always)]
    pub fn to_u16(self) -> u16 {
        self as u16
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::SnarkJobCommitmentPropagation => "snark_job_commitment/propagation",
        }
    }

    pub fn iter_all() -> impl Iterator<Item = ChannelId> {
        <Self as strum::IntoEnumIterator>::iter()
    }
}

impl std::fmt::Display for ChannelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_u8())
    }
}

#[derive(Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub struct MsgId(u64);

impl MsgId {
    pub fn first() -> Self {
        Self(1)
    }

    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, From, Debug, Clone)]
pub enum ChannelMsg {
    SnarkJobCommitmentPropagation(SnarkJobCommitmentPropagationChannelMsg),
}

impl ChannelMsg {
    pub fn channel_id(&self) -> ChannelId {
        match self {
            Self::SnarkJobCommitmentPropagation(_) => ChannelId::SnarkJobCommitmentPropagation,
        }
    }

    pub fn encode<W>(&self, w: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        match self {
            Self::SnarkJobCommitmentPropagation(v) => v.binprot_write(w),
        }
    }

    pub fn decode<R>(r: &mut R, id: ChannelId) -> Result<Self, binprot::Error>
    where
        Self: Sized,
        R: std::io::Read + ?Sized,
    {
        match id {
            ChannelId::SnarkJobCommitmentPropagation => {
                SnarkJobCommitmentPropagationChannelMsg::binprot_read(r).map(|v| v.into())
            }
        }
    }
}