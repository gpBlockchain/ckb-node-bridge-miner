build-ckbNodeBridge:
	git clone https://github.com/zhangsoledad/ckbNodeBridge.git
	cd ckbNodeBridge && cargo build

build-miner:
	cargo build

test:
	python test.py
