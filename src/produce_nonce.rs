use ckb_pow::pow_message;
use ckb_types::{U256};
use ckb_types::prelude::Pack;
use rdkafka::util::Timeout;
use tokio::spawn;
use log::*;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rand::prelude::*;
use eaglesong::eaglesong;
use crate::job::{MiningJob, SolvedShare};


pub fn produce_job(producer: FutureProducer, topic: String, job: MiningJob) {
    let ss = miner(job);
    match serde_json::to_string(&ss) {
        Ok(job_str) => {
            spawn(async move {
                info!("Producing mining job: {}", &job_str);
                let record = FutureRecord::to(topic.as_str())
                    .key("")
                    .partition(0)
                    .payload(&job_str);
                match producer.send(record, Timeout::Never).await {
                    Ok((partition, offset)) => {
                        info!(
                            "Mining job sent to partition {} offset {}",
                            partition, offset
                        );
                    }
                    Err((e, _)) => {
                        error!("Failed to produce mining job: {}", e);
                    }
                }
            });
        }
        Err(e) => {
            error!("Failed to serialize mining job: {}", e);
        }
    }
}

fn miner(job: MiningJob) -> SolvedShare {
    let mut rng = thread_rng();
    loop {
        let nonce = rng.gen_range(0, u128::MAX);
        let input = pow_message(&job.pow_hash.pack(), nonce);
        let mut output = [0u8; 32];
        eaglesong(&input, &mut output);
        // verify
        let target = U256::from_big_endian(&output[..]).expect("bound checked");
        if target <= job.target {
            info!("calc:{},nonce target:{}",target,job.target);
            return SolvedShare {
                work_id: job.work_id,
                height: job.height,
                timestamp: job.timestamp,
                pow_hash: job.pow_hash,
                target: job.target,
                nonce: nonce,
                job_id: 1,
                user_id: 0,
                worker_id: 0,
                worker_name: "ckb-miner-tool".to_string(),
            };
        }
    }
}