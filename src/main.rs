use crate::job::{consumer_job, hand_msg, Message};
use tokio::time::delay_for;
use std::time::Duration;

mod r#const;
mod job;

fn main() {
    env_logger::init();
    println!("Hello, world!");
    tokio_compat::run_std(async move {
        let (mut tx, rx) = tokio::sync::mpsc::channel::<Message>(256);
        hand_msg(
            rx
        );
        consumer_job(tx);
        delay_for(Duration::from_secs(36000000)).await;
    })
}
