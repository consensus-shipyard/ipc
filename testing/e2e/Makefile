# Functions to help set up the infrastructure needed to run end-to-end tests.

# Parameters that can be set when calling make to act like named arguments.
# e.g. `make <target> IPC_AGENT_NR=1`
IPC_AGENT_NR     := 0
IPC_NODE_NR      := 0
IPC_NODE_TYPE    := eudico
IPC_SUBNET_ID    := /root
IPC_SUBNET_NAME  := ""

PROJECT_ROOT     := ../..
IPC_INFRA_DIR    := $(PROJECT_ROOT)/bin/ipc-infra
IPC_SRC          := $(shell find $(PROJECT_ROOT) -type f \( -name "*.rs" \) | grep -v target)
IPC_AGENT        := $(PROJECT_ROOT)/bin/ipc-agent

IPC_NETWORK      := ipc
IPC_VERSION      := latest

# Location of artifacts derived from input parameters.
IPC_AGENT_DIR    := .ipc/agents/agent-$(IPC_AGENT_NR)
IPC_AGENT_CONFIG := $(IPC_AGENT_DIR)/config.toml
IPC_AGENT_SUBNETS:= $(shell find . -type f -wholename "./$(IPC_AGENT_DIR)/subnets/node-*")
IPC_NODE_DIR     := .ipc/nodes/node-$(IPC_NODE_NR)

# Find created agents and nodes
IPC_AGENTS       := $(shell find . -type d -wholename "./.ipc/agents/agent-*" | awk -F '/' '{print $$4}')
IPC_NODES        := $(shell find . -type d -wholename "./.ipc/nodes/node-*" | awk -F '/' '{print $$4}')


.PHONY: clean
clean: down
	docker network rm $(IPC_NETWORK) || exit 0
	rm -rf .ipc .make


# Stop and remove all nodes and agents.
down: \
		$(patsubst %, %/down, $(IPC_AGENTS)) \
		$(patsubst %, %/down, $(IPC_NODES))


# Create composition directory agent-$(IPC_AGENT_NR)
agent: $(IPC_AGENT_DIR) $(IPC_AGENT_CONFIG)


# Start an agent daemon.
agent/up: agent .make/docker/network .make/docker/images/ipc-agent
	cd $(IPC_AGENT_DIR) && docker-compose up -d


# Stop an agent daemon and remove its configuration, as if it never existed.
agent/down:
	if [ -d $(IPC_AGENT_DIR) ]; then \
		cd $(IPC_AGENT_DIR) && docker-compose down && cd - && rm -rf $(IPC_AGENT_DIR); \
	fi


# Stop and remove specific agent.
agent-%/down:
	$(MAKE) agent/down IPC_AGENT_NR=$*


# Reload the configuration of an agent.
agent/reload: agent/up
	$(IPC_AGENT) config reload --ipc-agent-url http://127.0.0.1:303$(IPC_AGENT_NR)/json_rpc


# Create composition directory node-$(IPC_NODE_NR)
node: $(IPC_NODE_DIR)


# Start a node.
node/up: node .make/docker/network
	cd $(IPC_NODE_DIR) && docker-compose up -d


# Stop a node and remove its configuration, as if it never existed.
node/down:
	if [ -d $(IPC_NODE_DIR) ]; then \
		cd $(IPC_NODE_DIR) && docker-compose down -v && cd - && rm -rf $(IPC_NODE_DIR); \
	fi


# Stop and remove a specific node.
node-%/down:
	$(MAKE) node/down IPC_NODE_NR=$*


# Connect an agent to a node.
# A node will always be participating in a specific subnet, so this means exactly one entry in the agent's config.
# Example: `make connect IPC_AGENT_NR=0 IPC_NODE_NR=0`
connect: node/up agent/up
	./scripts/connect.sh $(IPC_AGENT_DIR) $(IPC_NODE_DIR)
	$(MAKE) agent/reload


# Build the IPC agent binary.
$(IPC_AGENT): $(IPC_SRC)
	$(MAKE) -C $(PROJECT_ROOT) build


# Make sure the eudico docker image is built.
$(IPC_INFRA_DIR):
	$(MAKE) -C $(PROJECT_ROOT) install-infra


# Initialize the agent directory.
$(IPC_AGENT_DIR):
	mkdir -p $@
	cp template/agent/compose.yaml $@/compose.yaml

	@# The config.toml will be recreated from config.toml.orig plus the active subnets.
	cp template/agent/config.toml  $@/config.toml.orig

	@# Create .env file with overrides taken from the environment.
	export IPC_VERSION=$(IPC_VERSION) ; \
	export IPC_NETWORK=$(IPC_NETWORK) ; \
	export IPC_AGENT_NR=$(IPC_AGENT_NR) ; \
	./scripts/make-env.sh template/agent/.env $@/.env IPC_


# Initialize the agent config.
$(IPC_AGENT_CONFIG): $(IPC_AGENT_DIR) $(IPC_AGENT_SUBNETS)
	echo "# DO NOT EDIT! Any changes will be overwritten as subnets are added. Add changes to config.toml.orig instead." > $@
	cat $(IPC_AGENT_DIR)/config.toml.orig >> $@
	for subnet in $(IPC_AGENT_SUBNETS); do cat $$subnet >> $@; done


# Initialize the node directory based on the active node type.
$(IPC_NODE_DIR): | .make/$(IPC_NODE_TYPE)
	mkdir -p $@
	cp template/$(IPC_NODE_TYPE)/compose.yaml $@/

	@# Create .env file with overrides taken from the environment.
	export IPC_PROJECT_ROOT=${PWD}/$(PROJECT_ROOT) ; \
	export IPC_NETWORK=$(IPC_NETWORK) ; \
	export IPC_NODE_NR=$(IPC_NODE_NR) ; \
	export IPC_NODE_TYPE=$(IPC_NODE_TYPE) ; \
	export IPC_SUBNET_ID=$(IPC_SUBNET_ID) ; \
	export IPC_SUBNET_NAME=$(IPC_SUBNET_NAME) ; \
	./scripts/make-env.sh template/$(IPC_NODE_TYPE)/.env $@/.env IPC_


# Create the docker network every container connects to.
.make/docker/network:
	docker network create $(IPC_NETWORK)
	mkdir -p $(dir $@) && touch $@


# Build the IPC agent docker image.
.make/docker/images/ipc-agent: $(IPC_SRC)
	cd $(PROJECT_ROOT) && docker build -t ipc-agent .
	mkdir -p $(dir $@) && touch $@


# Build eudico specific artifacts
.make/eudico: $(IPC_INFRA_DIR)
