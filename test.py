import subprocess
import requests
import time

def get_tip_block_number(url):
    payload = {
        "jsonrpc": "2.0",
        "method": "get_tip_block_number",
        "params": [],
        "id": 64
    }

    headers = {
        'Content-Type': 'application/json'
    }

    try:
        response = requests.post(url, json=payload, headers=headers)
        if response.status_code == 200:
            result = response.json()
            if 'result' in result:
                return result['result']
            else:
                return None
        else:
            print(f"请求失败，状态码: {response.status_code}")
            return None
    except requests.RequestException as e:
        print(f"请求发生错误: {e}")
        return None


# 启动 docker-compose
subprocess.run("cd docker && docker-compose up > docker-compose.log 2>&1 &", shell=True)

# 启动 nodebridge
nodebridge_command = "RUST_LOG=info ./ckbNodeBridge/target/debug/nodebridge ckb --rpc_addr http://127.0.0.1:8114 --rpc_interval 3000 --kafka_brokers 127.0.0.1:9092 --db_url mysql://root:123456@127.0.0.1:8006/demo --job_topic tominer --solved_share_topic tochain > ckbNodeBridge.log 2>&1 &"
subprocess.run(nodebridge_command, shell=True, executable='/bin/bash')

# 启动 ckb-node-bridge-miner
ckb_miner_command = "./target/debug/ckb-node-bridge-miner --delay_s 1000 --job_topic tominer --kafka_url 127.0.0.1:9092 --solved_share_topic tochain >ckb-node-bridge-miner.log 2>&1 &"
subprocess.run(ckb_miner_command, shell=True)

time.sleep(60)

# 查询mysql数据库表found_blocks的所有信息
docker_compose_path = "docker"
mysql_command = f"docker-compose exec -T mysql mysql -uroot -p123456 -hmysql -P3306 demo -e 'SELECT * FROM found_blocks;'"
result = subprocess.check_output(mysql_command, shell=True, cwd=docker_compose_path)
print(result.decode('utf-8'))
tip_number = get_tip_block_number("http://localhost:8114/")
assert int(tip_number, 16) != 0
