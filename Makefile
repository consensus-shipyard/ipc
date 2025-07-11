# Each major sub-repository in this monorepo has their own Makefiles;
# instead of making an even more compilicated common one, let's delegate to them.

build: 
	cargo build --timings
	./target/debug/ipc-cli --version
	./target/debug/fendermint --version

default:
	# to be removed, only needed for unit-tests and end2end tests which don't have pnpm installed right now
	cd contracts && make gen
	cargo build --release --timings
	./target/release/ipc-cli --version
	./target/release/fendermint --version

SUBTREES_RUST := fendermint ipc ipld/resolver
SUBTREES_CONTRACTS := contracts
SUBTREES_ALL := $(SUBTREES_RUST) $(SUBTREES_CONTRACTS)

test: test-rust test-contracts

test-rust:
	cargo test --release --workspace

test-contracts: $(patsubst %, test/%, $(SUBTREES_CONTRACTS))

# Using `cd` instead of `-C` so $(PWD) is correct.
test/%:
	cd $* && make test

lint/%:
	cd $* && make lint || echo "$* lint failed"

license:
	./scripts/add_license.sh

lint: license $(patsubst %, lint/%, $(SUBTREES_ALL))

markdownlint:
	$(MARKDOWNLINT_CLI) --fix $$(find . -iwholename './crates/**/README.md' -or -iwholename './contracts/**/*.md' -or -iwholename './specs/**/*.md' -or -iwholename './docs*/**/*.md')

fmt:
	cargo +nightly fmt

check-fmt:
	cargo +nightly fmt --check

clippy:
	cargo clippy --workspace --tests --no-deps --fix --allow-dirty --allow-staged -- -D clippy::all

check-clippy:
	cargo clippy --workspace --tests --no-deps -- -D clippy::all


.PHONY: clippy check-clippy fmt check-fmt markdownlint license test test-rust test-contracts
