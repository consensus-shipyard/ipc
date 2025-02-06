# Each major sub-repository in this monorepo has their own Makefiles;
# instead of making an even more compilicated common one, let's delegate to them.

default:
	cd contracts && make gen
	cd crates && (cargo build --release && ./target/release/ipc-cli --version && ./target/release/fendermint --version)

# podman has a compatible API, and is better supported
CONTAINER_FRONTEND_BIN ?= docker
SUBTREES_RUST := $(patsubst %, crates/%, $(ls -1 crates))
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
