use std::str::from_utf8;
use std::collections::HashSet;
use ckb_pow::pow_message;
use serde::{Deserialize, Serialize};
use ckb_types::{H256, U256};
use ckb_types::prelude::Pack;
use rdkafka::{ClientConfig, TopicPartitionList};
use rdkafka::message::Message as KafkaMessage;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::util::Timeout;
use crate::r#const::{KAFKA_JOB_TOPIC, KAFKA_SOLVED_SHARE_TOPIC, KAFKA_URL};
use rdkafka::topic_partition_list::Offset;
use tokio::spawn;
use log::*;
use futures::StreamExt;
use futures::compat::Future01CompatExt;
use tokio::sync::mpsc::{Receiver, Sender};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rand::prelude::*;
use eaglesong::eaglesong;


pub enum Message {
    MiningJob(MiningJob),
}

impl From<MiningJob> for Message {
    fn from(block_template: MiningJob) -> Self {
        Message::MiningJob(block_template)
    }
}


#[derive(Clone, Serialize, Deserialize)]
pub struct MiningJob {
    pub work_id: u64,
    pub height: u64,
    pub timestamp: u64,
    pub target: U256,
    pub parent_hash: H256,
    pub pow_hash: H256,
}

pub fn consumer_job(mut tx: Sender<Message>) {
    spawn(async move {

        // 消费 kafka.job
        let topic = KAFKA_JOB_TOPIC;
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", KAFKA_URL)
            .set("enable.auto.commit", "false")
            .set("enable.partition.eof", "false")
            .set("group.id", "nodebridge-ckb")
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Failed to solved share consumer");

        // 获取消费信息转成 MiningJob -> send to produce_job
        let mut tpl = TopicPartitionList::new();
        tpl.add_partition_offset(&topic, 0, Offset::Offset(-2003));
        consumer
            .assign(&tpl)
            .expect("Failed to assign solved share topic");
        let mut stream = consumer.start();
        let mut sent_pow_hashes: HashSet<H256> = HashSet::new();
        while let Some(result) = stream.next().await {
            match result {
                Ok(msg) => match msg.detach().payload() {
                    Some(payload) => {
                        info!(
                            "Solved share received: {}",
                            from_utf8(payload).unwrap_or_default()
                        );
                        match serde_json::from_slice::<MiningJob>(payload) {
                            Ok(solved_share) => {
                                // todo solved_share.pow_hash already send
                                if !sent_pow_hashes.contains(&solved_share.pow_hash) {
                                    sent_pow_hashes.insert(solved_share.pow_hash.clone());
                                    tx.send(Message::from(solved_share))
                                        .await
                                        .unwrap_or_else(|e| {
                                            error!("Failed to send solved share for processing: {}", e);
                                        });
                                } else {
                                    info!("already send pow hash:{}",&solved_share.pow_hash)
                                }
                            }
                            Err(e) => {
                                error!("Failed to deserialize solved share: {}", e);
                            }
                        }
                    }
                    None => {
                        error!("No payload for solved share message");
                    }
                },
                Err(e) => {
                    warn!("Error while receiving from Kafka: {:?}", e);
                }
            }
        }
    });
}


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

pub fn hand_msg(mut rx: Receiver<Message>) {
    // 推送 SolvedShare
    // 获取 job 信息, 计算nonce ， 推送 到kafka.produce
    spawn(async move {
        info!(
            "Creating job producer to Kafka broker {} topic {}",
            KAFKA_URL, KAFKA_SOLVED_SHARE_TOPIC
        );
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", KAFKA_URL)
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Failed to create job producer");

        while let Some(msg) = rx.next().await {
            match msg {
                Message::MiningJob(block_template) => {
                    produce_job(
                        producer.clone(),
                        KAFKA_SOLVED_SHARE_TOPIC.to_string(),
                        block_template,
                    );
                }
            }
        }
    });
}

fn produce_job(producer: FutureProducer, topic: String, job: MiningJob) {
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
    return SolvedShare {
        work_id: job.work_id,
        height: job.height,
        timestamp: job.timestamp,
        pow_hash: job.pow_hash,
        target: job.target,
        nonce: 0,
        job_id: 1,
        user_id: 0,
        worker_id: 0,
        worker_name: "ckb-miner-tool".to_string(),
    };
}