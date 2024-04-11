FENDERMINT_DIR   := $(PWD)/../../..
SCRIPTS_DIR      := $(PWD)/scripts

TESTNET_ID       := state-size
MANIFEST_FILE    := $(PWD)/$(TESTNET_ID).yaml
NODE_ID          := bench

# Using the common materializer directory so port ranges are kept track of.
MATERIALIZER_DIR := $(FENDERMINT_DIR)/testing/materializer/tests/docker-materializer-data/
ROCKSDB_DIR      := $(MATERIALIZER_DIR)/testnets/$(TESTNET_ID)/root/nodes/$(NODE_ID)/fendermint/data/rocksdb

# By default test with keeping all state.
STATE_HIST_SIZE  ?= 0

# Measurement settings
MEASUREMENTS_PERIOD_SECS := 60
MEASUREMENTS_FILE        := $(PWD)/measurements.jsonline


# Export all the above variables as env vars, so they are available to scripts.
export

# Create the manifest file, e.g.:
# make manifest STATE_HIST_SIZE=100
manifest:
	$(SCRIPTS_DIR)/make-manifest.sh

# Setup the test node.
setup: manifest
	cd $(FENDERMINT_DIR) && \
	cargo run -q -p fendermint_app -- \
		materializer \
			--data-dir $(MATERIALIZER_DIR) \
			setup \
				--manifest-file $(MANIFEST_FILE)

# Teardown the test node.
teardown:
	cd $(FENDERMINT_DIR) && \
	cargo run -q -p fendermint_app -- \
		materializer \
			--data-dir $(MATERIALIZER_DIR) \
			teardown --testnet-id $(TESTNET_ID)

# Take periodic measurements.
measurements:
	$(SCRIPTS_DIR)/make-measurements.sh
