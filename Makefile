build-ckbNodeBridge:
	git clone https://github.com/gpBlockchain/ckbNodeBridge.git
	cd ckbNodeBridge && git checkout lgp/update-rust && cargo build

build-miner:
	cargo build

test:
	python test.py