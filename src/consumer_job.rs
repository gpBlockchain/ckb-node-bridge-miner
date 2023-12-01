use std::str::from_utf8;
use std::collections::HashSet;
use ckb_types::{H256};
use rdkafka::{ClientConfig, TopicPartitionList};
use rdkafka::message::Message as KafkaMessage;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::topic_partition_list::Offset;
use tokio::spawn;
use log::*;
use futures::StreamExt;
use tokio::sync::mpsc::{Sender};
use crate::job::{Message, MiningJob};


pub fn consumer_job(kafaka_url: String, topic: String, mut tx: Sender<Message>) {
    spawn(async move {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", kafaka_url.as_str())
            .set("enable.auto.commit", "false")
            .set("enable.partition.eof", "false")
            .set("group.id", "nodebridge-ckb")
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Failed to solved share consumer");

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
