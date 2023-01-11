# Tendermint

To implement the [architecture](./architecture.md) we intend to make use of the following open source components to integrate with Tendermint:

* [Tendermint Core](https://github.com/tendermint/tendermint): The generic blockchain SMR system. In particular we shall use the upcoming [v0.37](https://github.com/tendermint/tendermint/tree/v0.37.0-rc2) version which has the required [extensions](./architecture.md#abci) to [ABCI++](https://github.com/tendermint/tendermint/tree/v0.37.0-rc2/spec/abci). Note that the `tendermint/tendermint` repo is going to be archived; in the future it's possibly going to be developed further at https://github.com/informalsystems/tendermint, and further derivations will be registered at https://github.com/tendermint/ecosystem
* [tendermint-rs](https://github.com/informalsystems/tendermint-rs/) is a Rust library that contains Tendermint Core [datatypes](https://github.com/informalsystems/tendermint-rs/tree/main/tendermint); the [proto](https://github.com/informalsystems/tendermint-rs/tree/main/proto) code [generated](https://github.com/informalsystems/tendermint-rs/tree/main/tools/proto-compiler) from the Tendermint protobuf definitions; a synchronous [ABCI server](https://github.com/informalsystems/tendermint-rs/tree/main/abci) with a trait the application can implement, with a [KV-store example](https://github.com/informalsystems/tendermint-rs/blob/main/abci/src/application/kvstore/main.rs) familiar from the tutorial; and various other goodies for building docker images, integration testing the application with Tendermint, and so on. Lucky for us there is a [draft PR](https://github.com/informalsystems/tendermint-rs/pull/1193) to compile the protobuf definitions for both the current `v0.34` and the upcoming `v0.37` version, so we can even use that branch as a dependency to get the right data types and not have to do any proto compilation on our end!

Another project worth looking at is Penumbra's [tower-abci](https://github.com/penumbra-zone/tower-abci) which adapts the ABCI interfaces from `tendermint-rs` to be used with [tower](https://crates.io/crates/tower) and has a [server](https://github.com/penumbra-zone/tower-abci/blob/main/src/server.rs) implementation that works with [tokio](https://crates.io/crates/tokio). So, unlike the ABCI server in `tendermint-rs`, this is asynchronous; even if we don't use it, it's easy to follow as an example.

That should be enough to get us started with Tendermint.


## Install Tendermint Core

We will need Tendermint Core running and building the blockchain, and since we don't want to fork it, we can install the pre-packaged `tendermint` binary from the [releases](https://github.com/tendermint/tendermint/releases). At the time of this writing, our target is the [v0.37.0-rc2](https://github.com/tendermint/tendermint/releases/tag/v0.37.0-rc2) pre-release.

Alternatively, we can [install](https://github.com/tendermint/tendermint/blob/main/docs/introduction/install.md) the project from source. I expect to have to dig around in the source code to understand the finer nuances, so this is what I'll do. It needs `go` 1.18 or higher [installed](https://go.dev/doc/install) (check with `go version`).

The following code downloads the source, checks out the branch with the necessary ABCI++ features, and installs it.
```shell
git clone https://github.com/tendermint/tendermint.git
cd tendermint
git checkout v0.37.0-rc2
make install
```

Check that the installation worked:

```console
$ tendermint version
v0.37.0-rc2
```

After this we can follow the [quick start guide](https://github.com/tendermint/tendermint/blob/main/docs/introduction/quick-start.md#initialization) to initialize a local node and try out the venerable `kvstore` application.

Create the genesis files under `$HOME/.tendermint`:

```shell
tendermint init
```

Start a node; we'll see blocks being created every second:

```shell
tendermint node --proxy_app=kvstore
```

Then, from another terminal, send a transaction:

```shell
curl -s 'localhost:26657/broadcast_tx_commit?tx="foo=bar"'
```

Finally, query the value of the key we just added:

```console
$ curl -s 'localhost:26657/abci_query?data="foo"' | jq -r ".result.response.value | @base64d"
bar
```

Nice! The status of the node can be checked like so:

```shell
curl -s localhost:26657/status
```

To start from a clean slate, we can just clear out the data directory and run `tendermint init` again:

```shell
rm -rf ~/.tendermint
```

## Sanity check the Rust libraries

This is an optional step to check that the branch that we'll need to be using from `tendermint-rs` works with our chosen version of `tendermint`. In practice we'll just add a library reference to the github project until it's released, we don't have to clone the project. But it's useful to do so, to get familiar with the code.

```shell
git clone git@github.com:informalsystems/tendermint-rs.git
cd tendermint-rs
git checkout origin/mikhail/multi-tc-version-support
```

Then, go into the `abci` crate to try the [example](https://github.com/informalsystems/tendermint-rs/tree/main/abci#examples) with the `kvstore` that, unlike previously, will run external to `tendermint`:

```shell
cd abci
```

Build and run the store:

```shell
cargo run --bin kvstore-rs --features binary,kvstore-app
```

Go back to the terminal we used to run `tendermint` and do what they suggest.

First ensure we have the genesis files:

```shell
tendermint init
```

Then try to run Tendermint; it's supposed to connect to `127.0.0.1:26658` where the store is running, and bind itself to `127.0.0.1:26657`:

```shell
tendermint unsafe_reset_all && tendermint start
```

Unfortunately this doesn't seem to work. In the Tendermint logs we see the process stopping with an error message:

```console
$ tendermint start
I[2023-01-11|10:30:13.757] service start                                module=proxy msg="Starting multiAppConn service" impl=multiAppConn
I[2023-01-11|10:30:13.757] service start                                module=abci-client connection=query msg="Starting socketClient service" impl=socketClient
...
E[2023-01-11|10:30:13.777] Stopping abci.socketClient for error: read message: read tcp 127.0.0.1:35778->127.0.0.1:26658: read: connection reset by peer module=abci-client connection=query
I[2023-01-11|10:30:13.777] service stop                                 module=abci-client connection=query msg="Stopping socketClient service" impl=socketClient
E[2023-01-11|10:30:13.777] query connection terminated. Did the application crash? Please restart tendermint module=proxy err="read message: read tcp 127.0.0.1:35778->127.0.0.1:26658: read: connection reset by peer"
fish: Job 1, 'tendermint start' terminated by signal SIGTERM (Polite quit request)
```

We can see the opposite side of the error in the console of the store:

```console
$ cargo run --bin kvstore-rs --features binary,kvstore-app
   ...
2023-01-11T10:30:13.757461Z  INFO tendermint_abci::server: Incoming connection from: 127.0.0.1:35778
2023-01-11T10:30:13.757808Z  INFO tendermint_abci::server: Incoming connection from: 127.0.0.1:35792
2023-01-11T10:30:13.758160Z  INFO tendermint_abci::server: Incoming connection from: 127.0.0.1:35808
2023-01-11T10:30:13.758581Z  INFO tendermint_abci::server: Incoming connection from: 127.0.0.1:35816
2023-01-11T10:30:13.759077Z  INFO tendermint_abci::server: Listening for incoming requests from 127.0.0.1:35816
2023-01-11T10:30:13.759632Z  INFO tendermint_abci::server: Listening for incoming requests from 127.0.0.1:35778
2023-01-11T10:30:13.760138Z  INFO tendermint_abci::server: Listening for incoming requests from 127.0.0.1:35792
2023-01-11T10:30:13.760395Z  INFO tendermint_abci::server: Listening for incoming requests from 127.0.0.1:35808
2023-01-11T10:30:13.777266Z ERROR tendermint_abci::server: Failed to read incoming request from client 127.0.0.1:35778: error encoding protocol buffer

Caused by:
    failed to decode Protobuf message: Request.value: buffer underflow

Location:
    /home/aakoshh/.cargo/registry/src/github.com-1ecc6299db9ec823/flex-error-0.4.4/src/tracer_impl/eyre.rs:10:9
2023-01-11T10:30:13.781317Z  INFO tendermint_abci::server: Client 127.0.0.1:35816 terminated stream
2023-01-11T10:30:13.781337Z  INFO tendermint_abci::server: Client 127.0.0.1:35808 terminated stream
2023-01-11T10:30:13.781351Z  INFO tendermint_abci::server: Client 127.0.0.1:35792 terminated stream
```

We can try it with the latest stable version of Tendermint:

```shell
cd tendermint
git checkout v0.34.24
make install
rm ~/.tendermint
tendermint init
```

With this version, Tendermint is able to start:

```console
❯ tendermint unsafe_reset_all && tendermint start
Deprecated: snake_case commands will be replaced by hyphen-case commands in the next major release
I[2023-01-11|10:51:31.856] Removed all blockchain history               module=main dir=/home/aakoshh/.tendermint/data
I[2023-01-11|10:51:31.870] Reset private validator file to genesis state module=main keyFile=/home/aakoshh/.tendermint/config/priv_validator_key.json stateFile=/home/aakoshh/.tendermint/data/priv_validator_state.json
I[2023-01-11|10:51:31.978] service start                                module=proxy msg="Starting multiAppConn service" impl=multiAppConn
I[2023-01-11|10:51:31.978] service start                                module=abci-client connection=query msg="Starting socketClient service" impl=socketClient
I[2023-01-11|10:51:31.978] service start                                module=abci-client connection=snapshot msg="Starting socketClient service" impl=socketClient
I[2023-01-11|10:51:31.978] service start                                module=abci-client connection=mempool msg="Starting socketClient service" impl=socketClient
I[2023-01-11|10:51:31.978] service start                                module=abci-client connection=consensus msg="Starting socketClient service" impl=socketClient
I[2023-01-11|10:51:31.978] service start                                module=events msg="Starting EventBus service" impl=EventBus
I[2023-01-11|10:51:31.978] service start                                module=pubsub msg="Starting PubSub service" impl=PubSub
I[2023-01-11|10:51:31.993] service start                                module=txindex msg="Starting IndexerService service" impl=IndexerService
I[2023-01-11|10:51:31.994] ABCI Handshake App Info                      module=consensus height=0 hash=00000000000000000000000000000000 software-version=0.1.0 protocol-version=1
I[2023-01-11|10:51:31.994] ABCI Replay Blocks                           module=consensus appHeight=0 storeHeight=0 stateHeight=0
I[2023-01-11|10:51:31.998] Completed ABCI Handshake - Tendermint and App are synced module=consensus appHeight=0 appHash=00000000000000000000000000000000
I[2023-01-11|10:51:31.998] Version info                                 module=main tendermint_version=v0.34.24 block=11 p2p=8
...
```

And we can see commits in the ABCI server:

```console
...
2023-01-11T10:51:31.979523Z  INFO tendermint_abci::server: Listening for incoming requests from 127.0.0.1:57168
2023-01-11T10:51:33.064188Z  INFO tendermint_abci::application::kvstore: Committed height 1
2023-01-11T10:51:34.091852Z  INFO tendermint_abci::application::kvstore: Committed height 2
...
```

So it looks like something changed during the `0.34` to `0.37` transition in the Protobuf encoding that the decoder doesn't expect. We'll have to find out what and fix it.
