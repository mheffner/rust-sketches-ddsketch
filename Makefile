clean:
	cargo clean

test:
	cargo test

test_logs:
	cargo test -- --nocapture

test_performance:
	cargo test --release test_performance -- --ignored --nocapture