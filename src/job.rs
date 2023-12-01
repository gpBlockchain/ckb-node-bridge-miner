use serde::{Deserialize, Serialize};
use ckb_types::{H256, U256};


// consumer
#[derive(Clone, Serialize, Deserialize)]
pub struct MiningJob {
    pub work_id: u64,
    pub height: u64,
    pub timestamp: u64,
    pub target: U256,
    pub parent_hash: H256,
    pub pow_hash: H256,
}

// produce
#[derive(Serialize, Deserialize)]
pub struct SolvedShare {
    pub work_id: u64,
    pub height: u64,
    pub timestamp: u64,
    pub pow_hash: H256,
    pub target: U256,
    pub nonce: u128,
    pub job_id: u64,
    #[serde(rename = "userId")]
    pub user_id: i32,
    #[serde(rename = "workerId")]
    pub worker_id: i64,
    #[serde(rename = "workerFullName")]
    pub worker_name: String,
}


pub enum Message {
    MiningJob(MiningJob),
}

impl From<MiningJob> for Message {
    fn from(block_template: MiningJob) -> Self {
        Message::MiningJob(block_template)
    }
}

