test:
	cargo test --release --package primitives --package "fil_*"

check-fmt:
	cargo +nightly fmt --all --check

check-clippy:
	cargo clippy --all --tests -- -D clippy::all


lint: \
	check-fmt \
	check-clippy
