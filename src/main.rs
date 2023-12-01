use tokio::time::delay_for;
use std::time::Duration;
use crate::consumer_job::consumer_job;
use crate::job::{Message};
use rdkafka::{ClientConfig};
use tokio::spawn;
use log::*;
use futures::StreamExt;
use tokio::sync::mpsc::{Receiver};
use rdkafka::producer::{FutureProducer};

mod job;
mod consumer_job;
mod produce_nonce;

use structopt::StructOpt;
use crate::produce_nonce::produce_job;

#[derive(StructOpt)]
#[structopt(
name = "ckb-node-bridge-miner",
about = "adapt ckb-node-bridge",
rename_all = "snake_case"
)]
#[structopt(name = "ckb", about = "Node bridge for CKB network")]
#[derive(Clone)]
struct Ckb {
    /// Kafka brokers in comma separated <host>:<port> format
    #[structopt(long, default_value = "localhost:9092")]
    kafka_url: String,
    /// delay miner time
    #[structopt(long, default_value = "3000")]
    delay_s: u64,
    #[structopt(long)]
    /// Job topic name
    #[structopt(long)]
    job_topic: String,
    /// Solved share topic name
    #[structopt(long)]
    solved_share_topic: String,
}

fn main() {
    env_logger::init();
    let ckb = Ckb::from_args();


    tokio_compat::run_std(async move {
        let (tx, rx) = tokio::sync::mpsc::channel::<Message>(256);
        hand_msg(
            ckb.kafka_url.clone(), ckb.solved_share_topic.clone(), rx,
        );
        consumer_job(ckb.kafka_url.clone(), ckb.job_topic.clone(), tx);
        delay_for(Duration::from_secs(ckb.delay_s)).await;
    })
}


pub fn hand_msg(kafka_url: String, produce_topic: String, mut rx: Receiver<Message>) {
    spawn(async move {
        info!(
            "Creating job producer to Kafka broker {} topic {}",
            kafka_url.to_string().as_str(), produce_topic.to_string().as_str()
        );
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", kafka_url.as_str())
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Failed to create job producer");

        while let Some(msg) = rx.next().await {
            match msg {
                Message::MiningJob(block_template) => {
                    produce_job(
                        producer.clone(),
                        produce_topic.to_string(),
                        block_template,
                    );
                }
            }
        }
    });
}
