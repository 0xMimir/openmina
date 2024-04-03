use std::{num::NonZeroUsize, time::Duration};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct P2pNetworkKademliaConfig {
    alpha: NonZeroUsize,
    k_param: NonZeroUsize,
    timeout: Duration
}

impl Default for P2pNetworkKademliaConfig {
    fn default() -> Self {
        unsafe {
            Self {
                alpha: NonZeroUsize::new_unchecked(3),
                k_param: NonZeroUsize::new_unchecked(20),
                timeout: Duration::from_secs(10)
            }
        }
    }
}

impl P2pNetworkKademliaConfig {
    pub fn alpha(&self) -> usize {
        self.alpha.into()
    }

    pub fn k_param(&self) -> usize {
        self.k_param.into()
    }

    pub fn timeout(&self) -> &Duration{
        &self.timeout
    }
}
