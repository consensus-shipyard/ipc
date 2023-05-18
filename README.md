# Fendermint

Fendermint is an effort to implement [IPC with Tendermint Core](https://docs.google.com/document/d/1cFoTdoRuYgxmWJia6K-b5vmEj-4MvyHCNvShZpyconU/edit#). There is a preliminary [roadmap](https://docs.google.com/spreadsheets/d/1eVwkHEPGNg0js8DKRDIX7sugf5JqbI9zRBddIqzJFfI/edit#gid=0) that lays out the tasks towards implementing subnets that run IPLD and FVM under the Filecoin rootnet, sharing components with the Lotus/Eudico based implementation.

## Docs

Please have a look in the [docs](./docs/README.md) to see an overview of the project, how to run the components, and previous demos.

## Testing

The following command runs unit and integration tests:

```bash
make test
```

while the next command builds docker images and runs an end-to-end test using the [SimpleCoin](./fendermint/rpc/examples/simplecoin.rs) example:

```bash
make e2e
```
