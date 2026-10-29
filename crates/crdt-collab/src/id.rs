use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct OpId {
    pub lamport: u64,
    pub client_id: u64,
    pub seq: u32,
}

impl OpId {
    pub const ROOT: Self = Self {
        lamport: 0,
        client_id: 0,
        seq: 0,
    };

    pub fn root() -> Self {
        Self::ROOT
    }

    pub fn new(lamport: u64, client_id: u64, seq: u32) -> Self {
        Self {
            lamport,
            client_id,
            seq,
        }
    }

    pub fn is_root(&self) -> bool {
        self.client_id == 0 && self.seq == 0 && self.lamport == 0
    }
}
