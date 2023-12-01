# ckb-node-bridge-miner
adapt ckbNodeBridge : https://github.com/zhangsoledad/ckbNodeBridge


### start docker-compose
- ckb:8114
    - version:112
- mysql:3306
    - database:demo
      - table:found_blocks
- kafka:9092
  - topic
    - tominer(--job_topic)
    - tochain(--solved_share_topic)
```angular2html
cd docker
docker-compose up 

```


### nodebridge
link: https://github.com/zhangsoledad/ckbNodeBridge

```shell
RUST_LOG=info ./nodebridge ckb --rpc_addr http://127.0.0.1:8114 --rpc_interval 3000 --kafka_brokers 127.0.0.1:9092 --db_url mysql://root:123456@127.0.0.1:8006/demo --job_topic tominer --solved_share_topic tochain
```

### ckb-node-bridge-miner
```shell
cargo build 
./target/debug/ckb-node-bridge-miner  --delay_s 1000 --job_topic tominer --kafka_url 127.0.0.1:9092 --solved_share_topic tochain
```

### todo 

-[ ] write docker-compose