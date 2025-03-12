# Each major sub-repository in this monorepo has their own Makefiles;
# instead of making an even more compilicated common one, let's delegate to them.

# these targets check internally if anything changed
.PHONY: crates contracts fmt

default:
	make fmt
	make contracts
	make crates

crates: contracts
	cargo build --locked --manifest-path ./crates/Cargo.toml --release
	./crates/target/release/ipc-cli --version
	./crates/target/release/fendermint --version

contracts:
	make -C contracts gen

fmt: 
	cd crates && cargo +nightly-2024-07-05 fmt --all
	# taplo fmt
 
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

markdownlint:
	$(MARKDOWNLINT_CLI) --fix $$(find . -iwholename './crates/**/README.md' -or -iwholename './contracts/**/*.md' -or -iwholename './specs/**/*.md' -or -iwholename './docs*/**/*.md')
