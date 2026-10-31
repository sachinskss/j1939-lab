.PHONY: test bench demo lint

test:
	cargo test --workspace
	cargo build -p j1939-cli --bin bam_demo
	python -m pip install -e "python/[test]"
	python -m pytest python/tests

bench:
	cargo bench -p j1939-tp
	cargo run -p j1939-cli --bin latency_report --release

demo:
	cargo run -p j1939-cli --bin bam_demo

lint:
	cargo fmt --all -- --check
	cargo clippy --workspace --all-targets -- -D warnings
	python -m compileall -q python
	python .github/scripts/plugin-vocab-check.py
