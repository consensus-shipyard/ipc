# Each major sub-repository in this monorepo has their own Makefiles;
# instead of making an even more compilicated common one, let's delegate to them.

default:
	cd contracts && make gen
	cargo build --locked --release
	./target/release/ipc-cli --version
	./target/release/fendermint --version

SUBTREES_RUST := fendermint ipc ipld/resolver
SUBTREES_CONTRACTS := contracts
SUBTREES_ALL := $(SUBTREES_RUST) $(SUBTREES_CONTRACTS)

test: test-rust test-contracts

test-rust: $(patsubst %, test/%, $(SUBTREES_RUST))

test-contracts: $(patsubst %, test/%, $(SUBTREES_CONTRACTS))

# Using `cd` instead of `-C` so $(PWD) is correct.
test/%:
	cd $* && make test

lint/%:
	cd $* && make lint || echo "$* lint failed"

license:
	./scripts/add_license.sh

lint: license $(patsubst %, lint/%, $(SUBTREES_ALL))

## Hoku

install:
	cd fendermint && make install && cargo install iroh-cli

config-local:
	./scripts/setup.sh

run-local-iroh:
	iroh --rpc-addr 0.0.0.0:4919 start

run-local-fendermint:
	./scripts/run_fendermint.sh

run-local-cometbft:
	./scripts/run_cometbft.sh
