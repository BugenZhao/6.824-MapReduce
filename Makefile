APP = app_wc

build:
	cargo build --release

seq: build
	cargo run --release --package sequential -- -a ${APP} -i inputs/*

dist: build clean
	make dist-coordinator &
	sleep 1
	make dist-workers
	sleep 1
	@echo ">>> DIFF"
	make merge
	make diff

dist-coordinator:
	cargo run --release --package distributed --bin coordinator -- -i inputs/* -r 10

dist-worker:
	mkdir -p out
	cargo run --release --package distributed --bin worker -- -a ${APP}

dist-workers:
	make dist-worker &
	make dist-worker &
	make dist-worker

clean:
	rm -f out/mr-*-*

merge:
	cd out && sort mr-out* | grep . > mr-all

diff:
	diff mr-out-0 out/mr-all
