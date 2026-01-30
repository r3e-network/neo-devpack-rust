use neo_devpack::prelude::*;
use neo_devpack::serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct DaoConfig {
    pub owner: NeoByteString,
    pub token: NeoByteString,
    pub quorum: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: i64,
    pub proposer: NeoByteString,
    pub target: NeoByteString,
    pub method: String,
    pub arguments: Vec<u8>,
    pub title: String,
    pub description: String,
    pub start_time: i64,
    pub end_time: i64,
    pub yes_votes: i64,
    pub no_votes: i64,
    pub executed: bool,
}
