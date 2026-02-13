# Each major sub-repository in this monorepo has their own Makefiles;
# instead of making an even more compilicated common one, let's delegate to them.

default:
	cd contracts && make gen
	cargo build --release
	./target/release/ipc-cli --version
	./target/release/fendermint --version

build-ui:
	cd ipc && make build-ui

build-with-ui:
	cd ipc && make build-with-ui
	./target/release/ipc-cli --version
	./target/release/fendermint --version

install: default
	@echo "Installing binaries to ~/.cargo/bin..."
	@mkdir -p ~/.cargo/bin
	@cp ./target/release/ipc-cli ~/.cargo/bin/ipc-cli
	@cp ./target/release/fendermint ~/.cargo/bin/fendermint
	@chmod +x ~/.cargo/bin/ipc-cli ~/.cargo/bin/fendermint
	@echo "Installation complete! Make sure ~/.cargo/bin is in your PATH."
	@echo "Installed: ipc-cli and fendermint"

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

markdownlint:
	$(MARKDOWNLINT_CLI) --fix $$(find . -iwholename './crates/**/README.md' -or -iwholename './contracts/**/*.md' -or -iwholename './specs/**/*.md' -or -iwholename './docs*/**/*.md')
