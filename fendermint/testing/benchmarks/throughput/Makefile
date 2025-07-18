FENDERMINT_DIR   := $(PWD)/../../../..
SCRIPTS_DIR      := $(PWD)/scripts
CONFIGS_DIR      := $(PWD)/configs
RESULTS_DIR      := $(PWD)/results

# Default test parameters
VALIDATORS       ?= 4
TX_TYPE         ?= transfer
DURATION        ?= 300s
RAMP_UP         ?= false
CONCURRENT_USERS ?= 100
TARGET_TPS      ?= 1000

# Test manifest and network settings
TESTNET_ID      := throughput-bench
MANIFEST_FILE   := $(TESTNET_ID).yaml
NETWORK         := testnet

# Materializer directory
MATERIALIZER_DIR := $(FENDERMINT_DIR)/testing/materializer/tests/docker-materializer-data/

# Export variables for scripts
export

.PHONY: all clean setup teardown throughput-test stress-test results

all: throughput-test

# Clean up previous test artifacts
clean:
	rm -rf $(RESULTS_DIR)/*
	rm -f $(MANIFEST_FILE)

# Set up test environment
setup: clean
	mkdir -p $(RESULTS_DIR)
	$(SCRIPTS_DIR)/generate-manifest.sh $(VALIDATORS) $(TX_TYPE) > $(MANIFEST_FILE)
	cd $(FENDERMINT_DIR) && \
	cargo run -q -p fendermint_app -- \
		materializer \
			--data-dir $(MATERIALIZER_DIR) \
			setup \
				--manifest-file $(PWD)/$(MANIFEST_FILE)

# Teardown test environment
teardown:
	cd $(FENDERMINT_DIR) && \
	cargo run -q -p fendermint_app -- \
		materializer \
			--data-dir $(MATERIALIZER_DIR) \
			teardown --testnet-id $(TESTNET_ID)

# Run basic throughput test
throughput-test: setup
	@echo "Running throughput test: $(VALIDATORS) validators, $(TX_TYPE) transactions, $(DURATION) duration"
	$(SCRIPTS_DIR)/run-throughput-test.sh $(VALIDATORS) $(TX_TYPE) $(DURATION) $(CONCURRENT_USERS) $(TARGET_TPS)
	$(SCRIPTS_DIR)/analyze-results.sh $(RESULTS_DIR)/throughput-$(VALIDATORS)v-$(TX_TYPE)-$(DURATION).json

# Run stress test to find maximum TPS
stress-test: setup
	@echo "Running stress test: $(VALIDATORS) validators, ramp_up=$(RAMP_UP), $(DURATION) duration"
	$(SCRIPTS_DIR)/run-stress-test.sh $(VALIDATORS) $(RAMP_UP) $(DURATION)
	$(SCRIPTS_DIR)/analyze-results.sh $(RESULTS_DIR)/stress-$(VALIDATORS)v-$(DURATION).json

# Run full benchmark suite
benchmark-suite:
	@echo "Running complete benchmark suite..."
	$(SCRIPTS_DIR)/run-benchmark-suite.sh

# Generate results and reports
results:
	$(SCRIPTS_DIR)/generate-report.sh $(RESULTS_DIR)

# Run single validator baseline
single-validator:
	$(MAKE) throughput-test VALIDATORS=1 TX_TYPE=transfer DURATION=300s

# Run multi-validator comparison
multi-validator:
	$(MAKE) throughput-test VALIDATORS=4 TX_TYPE=transfer DURATION=300s
	$(MAKE) throughput-test VALIDATORS=7 TX_TYPE=transfer DURATION=300s
	$(MAKE) throughput-test VALIDATORS=10 TX_TYPE=transfer DURATION=300s

# Run contract interaction tests
contract-tests:
	$(MAKE) throughput-test VALIDATORS=4 TX_TYPE=erc20 DURATION=300s
	$(MAKE) throughput-test VALIDATORS=4 TX_TYPE=deploy DURATION=120s
	$(MAKE) throughput-test VALIDATORS=4 TX_TYPE=contract_call DURATION=300s

# Long-running endurance test
endurance-test:
	$(MAKE) throughput-test VALIDATORS=4 TX_TYPE=transfer DURATION=3600s CONCURRENT_USERS=50

# Quick smoke test
smoke-test:
	$(MAKE) throughput-test VALIDATORS=1 TX_TYPE=transfer DURATION=30s CONCURRENT_USERS=10