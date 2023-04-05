# Running Fendermint

The commands are all executed by the `fendermint` binary, which is produced from the `fendermint_app` crate,
so we have two ways to run the program:
* `./target/debug/fendermint <args>` (or wherever it's been installed)
* `cargo run -p fendermint_app -- <args>`

## Genesis

The first step we need to do is create a Genesis file which we'll pass to Tendermint,
which will pass it to Fendermint through ABCI. This Genesis file will be in JSON format,
as that is the convention with Tendermint, but we could also encode it in CBOR if we wanted.

Ostensibly the Genesis content could be coming from the parent chain itself, so the child
subnet participants don't have to go through the manual steps below, but we still have these
options to start a standalone network.

In the following sections we will create a Genesis file for a network named `test`. Start by creating a directory to hold the files:

```shell
mkdir test-network
```

### Create a new Genesis file

First, create a new `genesis.json` file devoid of accounts and validators. The `--base-fee` here is completely arbitrary.

```shell
cargo run -p fendermint_app -- genesis --genesis-file test-network/genesis.json new --network-name test --base-fee 1000
```

We can check what the contents look like:

```console
$ cat test-network/genesis.json
{
  "timestamp": 1680101412,
  "network_name": "test",
  "network_version": 18,
  "base_fee": "1000",
  "validators": [],
  "accounts": []
}
```

### Create some keys

Next, let's create some cryptographic key pairs we want want to use either for accounts or validators at Genesis.

```shell
mkdir test-network/keys
for NAME in alice bob charlie dave; do
  cargo run -p fendermint_app -- key gen --out-dir test-network/keys --name $NAME;
done
```

Check that the keys have been created:

```console
$ ls test-network/keys
alice.pk  alice.sk  bob.pk  bob.sk  charlie.pk  charlie.sk  dave.pk  dave.sk

$ cat test-network/keys/alice.pk
Ak5Juk793ZAg/7Ojj4bzOmIFGpwLhET1vg2ROihUJFkq
```

### Add accounts to the Genesis file

Add one of the keys we created to the Genesis file as a stand-alone account:

```shell
 cargo run -p fendermint_app -- \
        genesis --genesis-file test-network/genesis.json \
        add-account --public-key test-network/keys/alice.pk --balance 1000000000000000000
```

Check that the balance is correct:

```console
$ cat test-network/genesis.json | jq .accounts
[
  {
    "meta": {
      "Account": {
        "owner": "f1jqqlnr5b56rnmc34ywp7p7i2lg37ty23s2bmg4y"
      }
    },
    "balance": "1000000000000000000"
  }
]
```

The `owner` we see is an `f1` type address with the hash of the public key. Technically it's an `Address` type,
but it has to be one based on a public key, otherwise we would not be able to validate signatures later.

Let's add an example of the other possible account type, a multi-sig account:

```shell
cargo run -p fendermint_app -- \
        genesis --genesis-file test-network/genesis.json \
        add-multisig --public-key test-network/keys/bob.pk --public-key test-network/keys/charlie.pk --public-key test-network/keys/dave.pk \
          --threshold 2 --vesting-start 0 --vesting-duration 1000000 --balance 3000000000000000000
```

Check that all three of the signatories have been added:

```console
$ cat test-network/genesis.json | jq .accounts[1]
{
  "meta": {
    "Multisig": {
      "signers": [
        "f1kgtzp5nuob3gdccagivcgns7e25be2c2rqozilq",
        "f1bvdmcbct6vwoh3rvkhoth2fe66p6prpbsziqbfi",
        "f1hgeqjtadqmyabmy2kij2smn5jiiud75kva6bzny"
      ],
      "threshold": 2,
      "vesting_duration": 1000000,
      "vesting_start": 0
    }
  },
  "balance": "3000000000000000000"
}
```

### Add validators to the Genesis file

Finally, let's add one validator to the Genesis, with a monopoly on voting power, so we can run a standalone node:

```shell
cargo run -p fendermint_app -- \
      genesis --genesis-file test-network/genesis.json \
      add-validator --public-key test-network/keys/bob.pk --power 1;
```

The value of power doens't matter in this case, as `bob` is our only validator. Check the result:

```console
$ cat test-network/genesis.json | jq .validators
[
  {
    "public_key": "BCImfwVC/LeFJN9bB612aCtjbCYWuilf2SorSUXez/QEy8cVKNuvTU/EOTibo3hIyOQslvSouzIpR24h1kkqCSI=",
    "power": 1
  },
]
```

The public key was spliced in as it was, in base64 format, which is how it would appear in Tendermint's
own genesis file format. Note that here we don't have the option to use `Address`, because we have to return
these as actual `PublicKey` types to Tendermint through ABCI, not as a hash of a key.

### Configure Tendermint

First, follow the instructions in [getting started with Tendermint](./tendermint.md) to install the binary,
then initialize a genesis file for Tendermint at `~/.tendermint`.

```shell
rm -rf ~/.tendermint
tendermint init
```

The logs show that it created keys and a genesis file:

```console
I[2023-03-29|09:58:06.324] Found private validator                      module=main keyFile=/home/aakoshh/.tendermint/config/priv_validator_key.json stateFile=/home/aakoshh/.tendermint/data/priv_validator_state.json
I[2023-03-29|09:58:06.324] Found node key                               module=main path=/home/aakoshh/.tendermint/config/node_key.json
I[2023-03-29|09:58:06.324] Found genesis file                           module=main path=/home/aakoshh/.tendermint/config/genesis.json
```

#### Convert the Genesis file

We don't want to use the random values created by Tendermint; instead we need to use some CLI commands to convert the artifacts
file we created earlier to the format Tendermint accepts. Start with the genesis file:

```shell
mv ~/.tendermint/config/genesis.json ~/.tendermint/config/genesis.json.orig
cargo run -p fendermint_app -- \
  genesis --genesis-file test-network/genesis.json \
  into-tendermint --out ~/.tendermint/config/genesis.json
```

Check the contents of the created Tendermint Genesis file:

<details>
  <summary>~/.tendermint/config/genesis.json</summary>

```console
$ cat ~/.tendermint/config/genesis.json
{
  "genesis_time": "2023-03-29T14:50:12Z",
  "chain_id": "test",
  "initial_height": "0",
  "consensus_params": {
    "block": {
      "max_bytes": "22020096",
      "max_gas": "-1",
      "time_iota_ms": "1000"
    },
    "evidence": {
      "max_age_num_blocks": "100000",
      "max_age_duration": "172800000000000",
      "max_bytes": "1048576"
    },
    "validator": {
      "pub_key_types": [
        "secp256k1"
      ]
    }
  },
  "validators": [],
  "app_hash": "",
  "app_state": {
    "accounts": [
      {
        "balance": "1000000000000000000",
        "meta": {
          "Account": {
            "owner": "f1jqqlnr5b56rnmc34ywp7p7i2lg37ty23s2bmg4y"
          }
        }
      },
      {
        "balance": "3000000000000000000",
        "meta": {
          "Multisig": {
            "signers": [
              "f1kgtzp5nuob3gdccagivcgns7e25be2c2rqozilq",
              "f1bvdmcbct6vwoh3rvkhoth2fe66p6prpbsziqbfi",
              "f1hgeqjtadqmyabmy2kij2smn5jiiud75kva6bzny"
            ],
            "threshold": 2,
            "vesting_duration": 1000000,
            "vesting_start": 0
          }
        }
      }
    ],
    "base_fee": "1000",
    "network_name": "test",
    "network_version": 18,
    "timestamp": 1680101412,
    "validators": [
      {
        "power": 1,
        "public_key": "BCImfwVC/LeFJN9bB612aCtjbCYWuilf2SorSUXez/QEy8cVKNuvTU/EOTibo3hIyOQslvSouzIpR24h1kkqCSI="
      }
    ]
  }
}
```

</details>

We can see that our original `genesis.json` has been made part of Tendermint's version under `app_state`,
and that the top level `validators` are empty, to be filled out by the application during the `init_chain` ABCI call.


#### Convert the private key

By default Tendermint uses Ed25519 validator keys, but in theory it can use anything that looks like a key.

We can run the following command to replace the default `priv_validator_key.json` file with one based on
one of the validators we created.

```shell
mv ~/.tendermint/config/priv_validator_key.json ~/.tendermint/config/priv_validator_key.json.orig
cargo run -p fendermint_app -- \
  key into-tendermint --secret-key test-network/keys/bob.sk --out ~/.tendermint/config/priv_validator_key.json
```

See if it looks reasonable:

<details>
<summary>~/.tendermint/config/priv_validator_key.json</summary>

```console
$ cat ~/.tendermint/config/priv_validator_key.json
{
  "address": "66FA0CFB373BD737DBFC7CE70BEF994DD42A3812",
  "priv_key": {
    "type": "tendermint/PrivKeySecp256k1",
    "value": "04Gsfaw4RHZ5hTbXO/3hz2N567tz5E1yxChM1ZrEi1E="
  },
  "pub_key": {
    "type": "tendermint/PubKeySecp256k1",
    "value": "AiImfwVC/LeFJN9bB612aCtjbCYWuilf2SorSUXez/QE"
  }
}
$ cat test-network/keys/bob.pk
AiImfwVC/LeFJN9bB612aCtjbCYWuilf2SorSUXez/QE
```
</details>

## Run processes

The Fendermint Application and Tendermint Core will run as separate processes.

### Run the Application

Now we are ready to start our Fendermint Application, which will listen to requests coming from Tendermint
through the ABCI interface.

First, let's make sure we have put all the necessary files in an easy to remember place under `~/.fendermint`.

```shell
mkdir -p ~/.fendermint/data
cp -r ./fendermint/app/config ~/.fendermint/config
```

We will need the actor bundle to load. We can configure its location via environment variables, but the default
configuration will look for it at `~/.fendermint/bundle.car`, so we might as well put it there.

```shell
make actor-bundle
cp ../builtin-actors/output/bundle.car ~/.fendermint/bundle.car
```

Now, start the application.

```shell
cargo run -p fendermint_app --release -- run
```

It is important to use the `--release` option, otherwise it will take too long to load the Wasm actor modules and
Tendermint will issue a timeout (by default we have 3 seconds to execute requests).

With the default `--log-level info` we can see that the application is listening:

```console
2023-03-29T09:17:28.548878Z  INFO fendermint::cmd: reading configuration path="/home/aakoshh/.fendermint/config"
2023-03-29T09:17:28.549700Z  INFO fendermint::cmd::run: opening database path="/home/aakoshh/.fendermint/data/rocksdb"
2023-03-29T09:17:28.879916Z  INFO tower_abci::server: starting ABCI server addr="127.0.0.1:26658"
2023-03-29T09:17:28.880023Z  INFO tower_abci::server: bound tcp listener local_addr=127.0.0.1:26658
```

If we need to restart the application from scratch, we can do so by erasing all RocksDB state:

```shell
rm -rf ~/.fendermint/data/rocksdb
```

### Run Tendermint

Tendermint can be configured via `~/.tendermint/config/config.toml`; see the default settings [here](https://docs.tendermint.com/v0.34/tendermint-core/configuration.html).

Now we are ready to start Tendermint and let it connect to the Fendermint Application.

```shell
tendermint start
```

If we need to restart the application from scratch, we can erase all Tendermint state like so:

```shell
tendermint unsafe-reset-all
```

If all goes well, we will see block created in the Fendermint Application log as well the Tendermint log:

<details>
  <summary>Application log</summary>

```console
$ rm -rf ~/.fendermint/data/rocksdb && cargo run -p fendermint_app --release -- --log-level debug run
...
2023-03-30T11:51:34.239909Z DEBUG tower_abci::server: new request request=Info(Info { version: "v0.37.0-rc2", block_version: 11, p2p_version: 8, abci_version: "1.0.0" })
...
2023-03-30T11:51:34.240250Z DEBUG tower_abci::server: flushing response response=Ok(Info(Info { data: "fendermint", version: "0.1.0", app_version: 0, last_block_height: block::Height(0), last_block_app_hash: AppHash(0171A0E402203AAAC8F10B0E837FDF2546C98BF164972B07B49196E25322711E3C4807CF8AD8) }))
2023-03-30T11:51:34.240914Z DEBUG tower_abci::server: new request request=InitChain(...)
...
2023-03-30T11:51:34.295133Z  INFO fendermint_app::app: init chain state_root="bafy2bzaceaurow7dd2zs2zek7jb44x4jumraubzy5fyjya5edgxnc32nhap76" app_hash="0171A0E4022029175BE31EB32D648AFA43CE5F89A3220A0738E9709C03A419AED16F4D381FFF"
2023-03-30T11:51:34.295665Z DEBUG tower_abci::server: flushing response response=Ok(InitChain(...))
...
2023-03-30T11:51:35.365180Z DEBUG tower_abci::server: new request request=BeginBlock(...)
...
2023-03-30T11:51:35.365662Z DEBUG fendermint_app::app: begin block height=1
2023-03-30T11:51:42.552711Z DEBUG fendermint_app::app: initialized exec state
2023-03-30T11:51:42.553013Z DEBUG tower_abci::server: flushing response response=Ok(BeginBlock(...))
...
2023-03-30T11:51:42.560459Z DEBUG tower_abci::server: flushing response response=Ok(Commit(...))
...
2023-03-30T11:51:42.606102Z DEBUG tower_abci::server: new request request=BeginBlock(...)
...
2023-03-30T11:51:42.606359Z DEBUG fendermint_app::app: begin block height=2
2023-03-30T11:51:42.606623Z DEBUG fendermint_app::app: initialized exec state
...
```
</details>


<details>
  <summary>Tendermint log</summary>

```console
$ tendermint unsafe-reset-all && tendermint start
...
I[2023-03-30|12:51:34.240] ABCI Handshake App Info                      module=consensus height=0 hash=0171A0E402203AAAC8F10B0E837FDF2546C98BF164972B07B49196E25322711E3C4807CF8AD8 software-version=0.1.0 protocol-version=0
I[2023-03-30|12:51:34.240] ABCI Replay Blocks                           module=consensus appHeight=0 storeHeight=0 stateHeight=0
I[2023-03-30|12:51:34.299] Completed ABCI Handshake - Tendermint and App are synced module=consensus appHeight=0 appHash=0171A0E402203AAAC8F10B0E837FDF2546C98BF164972B07B49196E25322711E3C4807CF8AD8
...
I[2023-03-30|12:51:35.335] received proposal                            module=consensus proposal="Proposal{1/0 (9FD634BC038D3CA4FC885E8530CD56B1693739AEBACBF404AAB5DDA5ADC8D180:1:756F1391A4CF, -1) 2BC2F835CBC1 @ 2023-03-30T11:51:35.327328663Z}"
I[2023-03-30|12:51:35.339] received complete proposal block             module=consensus height=1 hash=9FD634BC038D3CA4FC885E8530CD56B1693739AEBACBF404AAB5DDA5ADC8D180
I[2023-03-30|12:51:35.357] finalizing commit of block                   module=consensus height=1 hash=9FD634BC038D3CA4FC885E8530CD56B1693739AEBACBF404AAB5DDA5ADC8D180 root=0171A0E4022029175BE31EB32D648AFA43CE5F89A3220A0738E9709C03A419AED16F4D381FFF num_txs=0
I[2023-03-30|12:51:38.316] Timed out                                    module=consensus dur=3s height=1 round=0 step=RoundStepPropose
I[2023-03-30|12:51:42.553] executed block                               module=state height=1 num_valid_txs=0 num_invalid_txs=0
I[2023-03-30|12:51:42.560] committed state                              module=state height=1 num_txs=0 app_hash=0171A0E4022029175BE31EB32D648AFA43CE5F89A3220A0738E9709C03A419AED16F4D381FFF
I[2023-03-30|12:51:42.564] Timed out                                    module=consensus dur=-6.207267593s height=2 round=0 step=RoundStepNewHeight
I[2023-03-30|12:51:42.567] indexed block                                module=txindex height=1
I[2023-03-30|12:51:42.577] received proposal                            module=consensus proposal="Proposal{2/0 (5D2D09F6829D7F0481E597CEAE87DCFA5665987B0B7D57C05B302BEB8DB95406:1:4D3E5565CA22, -1) 693D8DF9C36E @ 2023-03-30T11:51:42.570865715Z}"
I[2023-03-30|12:51:42.581] received complete proposal block             module=consensus height=2 hash=5D2D09F6829D7F0481E597CEAE87DCFA5665987B0B7D57C05B302BEB8DB95406
I[2023-03-30|12:51:42.598] finalizing commit of block                   module=consensus height=2 hash=5D2D09F6829D7F0481E597CEAE87DCFA5665987B0B7D57C05B302BEB8DB95406 root=0171A0E4022029175BE31EB32D648AFA43CE5F89A3220A0738E9709C03A419AED16F4D381FFF num_txs=0
I[2023-03-30|12:51:42.607] executed block                               module=state height=2 num_valid_txs=0 num_invalid_txs=0
I[2023-03-30|12:51:42.612] committed state                              module=state height=2 num_txs=0 app_hash=0171A0E4022029175BE31EB32D648AFA43CE5F89A3220A0738E9709C03A419AED16F4D381FFF
I[2023-03-30|12:51:42.618] indexed block                                module=txindex height=2
...
```
</details>

Note that the first block execution is very slow because we have to load the Wasm engine, as indicated by the first proposal having a timeout,
but after that the blocks come in fast, one per second.


## Query the state

The Fendermint binary has some commands to support querying state. Behind the scenes it uses the `tendermint_rpc` crate to talk
to the Tendermint JSON-RPC endpoints, which forward the requests to the Application through ABCI.

Assuming both processes have been started, see if we can query the state of one of our actors. For this we need the actor address,
which we saw in the `genesis.json` file earlier.

```shell
cargo run -p fendermint_app --release -- \
  rpc query actor-state --address f1jqqlnr5b56rnmc34ywp7p7i2lg37ty23s2bmg4y | jq
```

The state is printed to STDOUT as JSON:

```console
{
  "id": 100,
  "state": {
    "balance": "1000000000000000000",
    "code": "bafk2bzaceaqi73ey2grdtwgmnl22yj37l6pttsmrqnoc44o6wdqtt5rmpam6y",
    "delegated_address": null,
    "sequence": 0,
    "state": "bafy2bzaceaayg22rmfjw5di7obchf2cdz3yydo32njavenej5uluf7hfatosi"
  }
}
```

What we see here is the general [ActorState](https://github.com/filecoin-project/builtin-actors/blob/v10.0.0/actors/account/src/state.rs) which contains the balance, the nonce, the Wasm code CID, and the state root hash of the
actual actor implementation, which in this case is an `Account` actor.

We can retrieve the raw state of the account with the `ipld` command:

```shell
cargo run -p fendermint_app --release -- \
        rpc query ipld --cid bafy2bzaceaayg22rmfjw5di7obchf2cdz3yydo32njavenej5uluf7hfatosi
```

The binary contents are printed with Base64 encoding, which we could pipe to a file. It would be more useful to run this query
programmatically and parse it to the appropriate data structure from [builtin-actors](https://github.com/filecoin-project/builtin-actors).

```console
gVUBTCC2x6HvotYLfMWf9/0aWbf541s=
```

## Transfer tokens

The simplest transaction we can do is to transfer tokens from one account to another.

For example we can send 1000 tokens from Alice to Bob:

```shell
BOB_ADDR=$(cargo run -p fendermint_app --release -- key address --public-key test-network/keys/bob.pk)
cargo run -p fendermint_app --release -- \
  rpc transfer --secret-key test-network/keys/alice.sk --to $BOB_ADDR --sequence 0 --value 1000 | jq
```

The `transfer` command waits for the commit results of the transaction:

```console
$ cargo run -p fendermint_app --release -- rpc transfer --secret-key test-network/keys/alice.sk --to $BOB_ADDR --sequence 0 --value 1000 | jq
    Finished dev [unoptimized + debuginfo] target(s) in 0.40s
     Running `target/debug/fendermint rpc transfer --secret-key test-network/keys/alice.sk --to f1kgtzp5nuob3gdccagivcgns7e25be2c2rqozilq --sequence 0 --value 1000`
{
  "response": {
    "check_tx": {
      "code": 0,
      "codespace": "",
      "data": null,
      "events": [],
      "gas_used": "0",
      "gas_wanted": "10000000000",
      "info": "",
      "log": "",
      "mempool_error": "",
      "priority": "0",
      "sender": "f1jqqlnr5b56rnmc34ywp7p7i2lg37ty23s2bmg4y"
    },
    "deliver_tx": {
      "code": 0,
      "codespace": "",
      "data": null,
      "events": [],
      "gas_used": "1124863",
      "gas_wanted": "10000000000",
      "info": "",
      "log": ""
    },
    "hash": "01828E0A350445ED3E8028D045EE99B5547B6834DB7296B799B95707EB546EC2",
    "height": "46"
  },
  "return_data": null
}
```

The `code: 0` parts indicate that both check and delivery were successful. Let's check the resulting states:

```console
$ cargo run -p fendermint_app --release -- rpc query actor-state --address $BOB_ADDR | jq .state.balance
"1000"

$ ALICE_ADDR=$(cargo run -p fendermint_app --release -- key address --public-key test-network/keys/alice.pk)

$ cargo run -p fendermint_app --release -- rpc query actor-state --address $ALICE_ADDR | jq "{balance: .state.balance, sequence: .state.sequence}"
{
  "balance": "999999999999999000",
  "sequence": 1
}
```

Great, Alice's nonce was correctly increased as well.


## Create FEVM Contract

When we want to deploy a smart contract to the FVM, the currently supported way is by deploying EVM contracts to FEVM.

First, we need the `solc` compiler to produce the binaries we want deployed; take a look at the [test contracts](https://github.com/filecoin-project/builtin-actors/tree/next/actors/evm/tests/contracts) in the `builtin-actors` repo for example.

Say we want to deploy the `SimpleCoin` contract from that directory.

```shell
CONTRACT=../builtin-actors/actors/evm/tests/contracts/SimpleCoin.bin
cargo run -p fendermint_app --release -- \
  rpc fevm --secret-key test-network/keys/alice.sk --sequence 0 \
    create --contract $CONTRACT | jq
```

The output shows what addresses have been assigned to the created contract,
which we can use to call the contract.

```console
$ cargo run -p fendermint_app --release -- \
        rpc fevm --secret-key test-network/keys/alice.sk --sequence 0 \
          create --contract $CONTRACT | jq .return_data
{
  "actor_address": "f0105",
  "actor_id": 105,
  "actor_id_as_eth_address": "ff00000000000000000000000000000000000069",
  "delegated_address": "f410fsho763qmlcfi6ufnim7sujmbaqyc64b3pzpa7bq",
  "eth_address": "91ddff6e0c588a8f50ad433f2a258104302f703b",
  "robust_address": "f2rd3cu2jokusukudmbwotuu4rvoebm45qnze7e6q"
}
```

## Invoke FEVM Contract

Now taht we have a contract deployed, we can call it. The following is just based on on the example in [fvm-bench](https://github.com/filecoin-project/fvm-bench). Normally we'd try to

```console
$ cargo run -p fendermint_app --release -- \
              rpc fevm --secret-key test-network/keys/alice.sk --sequence 1 \
                invoke --contract f410fsho763qmlcfi6ufnim7sujmbaqyc64b3pzpa7bq  \
                       --method f8b2cb4f --method-args 000000000000000000000000ff00000000000000000000000000000000000064 \
          | jq .return_data
"0000000000000000000000000000000000000000000000000000000000002710"
```
