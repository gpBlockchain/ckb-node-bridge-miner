# ckb-node-bridge-miner
adapt ckbNodeBridge : https://github.com/zhangsoledad/ckbNodeBridge


### start mysql  
```shell
docker run -d --name testmysql1122 -p 33060:3306 -e MYSQL_ROOT_PASSWORD=123456 mysql:5.7.23

mysql -u root -p

CREATE DATABASE demo;

CREATE TABLE found_blocks (
    id INT AUTO_INCREMENT PRIMARY KEY,
    puid INT,
    worker_id BIGINT,
    worker_full_name VARCHAR(255),
    height BIGINT,
    hash VARCHAR(255),
    hash_no_nonce VARCHAR(255),
    nonce VARCHAR(255),
    prev_hash VARCHAR(255),
    network_diff INT,
    created_at TIMESTAMP
); 
```
### start kafka
link: https://kafka.apache.org/downloads
```shell
./bin/zookeeper-server-start.sh config/zookeeper.properties
./bin/kafka-server-start.sh config/server.properties 
# create topic : solved_share_topic
./bin/kafka-console-consumer.sh --topic solved_share_topic --bootstrap-server localhost:9092 --from-beginning

```
### start ckb 
link: https://github.com/nervosnetwork/ckb.git
dev.toml 
```shell
[pow]
func = "Eaglesong"

```

### nodebridge
link: https://github.com/zhangsoledad/ckbNodeBridge

```shell

RUST_LOG=info ./nodebridge ckb --rpc_addr http://localhost:8114 --rpc_interval 3000 --kafka_brokers localhost:9092 --db_url mysql://root:123456@localhost:33060/demo --job_topic test --solved_share_topic solved_share_topic

```

### ckb-node-bridge-miner
```shell
cargo build 
./target/debug/ckb-node-bridge-miner
```

### todo 

-[ ] write docker-compose