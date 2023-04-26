# Integration tests

This directory includes a set of tools to perform end-to-end integration tests between the agent and the underlying subnet infrastructure.Before running the test cases, one needs to launch a `lotus` cluster and a `ipc-agent` daemon using the instructions shared in the project's [README](../../README.md).
Once the infrastructure has been setup, the integration tests can be run using:

```shell
cargo test -p ipc_e2e --test <TESTCASE_NAME>

# To run the subnet lifecycle test, perform:
cargo test -p ipc_e2e --test subnet_lifecycle
```

> Note: This is a basic skeleton to showcase how we can run automated end-to-end tests over IPC. In the future, the goal is to automate the deployment of the IPC agent and the infrastructure so all tests can be run automatically.


## Test environment

The `template` directory contains `docker-compose` files for creating a test environment with varying number of agents and nodes using the commands in the `Makefile`.

For example to start the default agent in `docker`, run the following:

```shell
make agent/up
```

All artifacts created during the procedure are stored under the `.ipc` directory, which has the following structure:

```console
❯ tree -a .ipc
.ipc
├── agents
│   └── agent-0
│       ├── compose.yaml
│       ├── config.toml
│       ├── config.toml.orig
│       ├── .env
│       └── subnets
│           └── node-0
└── nodes
    └── node-0
        ├── compose.yaml
        └── .env

5 directories, 7 files
```

There can be multiple agents, and their corresponding `config.toml` files will be generated as we create more nodes and subnets. To create another agent, we would run `make agent IPC_AGENT_NR=1`.

The main targets of the `Makefile` are:

* `make agent`: create a configuration directory for `$IPC_AGENT_NR`; the container isn't started yet, so we could make some modifications if necessary
* `make agent/up`: start the docker container for `$IPC_AGENT_NR`; if necessary build the `ipc-agent` docker image, the configuration directory, etc.
* `make agent/down`: remove the docker container for `$IPC_AGENT_NR` and the agent configuration directory
* `make node`, `make node/up`, `make node/down`: same as for the agent
* `make down`: stop and remove all agents and nodes
* `make connect`: connect `$IPC_AGENT_NR` to `$IPC_NODE_NR` and reload the agent configuration
