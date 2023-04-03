# Integration tests
This directory includes a set of tools to perform end-to-end integration tests between the agent and the underlying subnet infrastructure.Before running the test cases, one needs to launch a `lotus` cluster and a `ipc-agent` daemon using the instructions shared in the project's [README](../README).
Once the infrastructure has been setup, the integration tests can be run using:
```shell
cargo test --test <TESTCASE_NAME>

# To run the subnet lifecycle test, perform:
cargo test --test subnet_lifecycle
```

> Note: This is a basic skeleton to showcase how we can run automated end-to-end tests over IPC. In the future, the goal is to automate the deployment of the IPC agent and the infrastructure so all tests can be run automatically.
