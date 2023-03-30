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


### Run the application

Now we are ready to start our Fendermint _Application_, which will listen to requests coming from Tendermint
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
cargo run -p fendermint_app -- run
```

With the default `--log-level info` we can see that the application is listening:

```console
2023-03-29T09:17:28.548878Z  INFO fendermint::cmd: reading configuration path="/home/aakoshh/.fendermint/config"
2023-03-29T09:17:28.549700Z  INFO fendermint::cmd::run: opening database path="/home/aakoshh/.fendermint/data/rocksdb"
2023-03-29T09:17:28.879916Z  INFO tower_abci::server: starting ABCI server addr="127.0.0.1:26658"
2023-03-29T09:17:28.880023Z  INFO tower_abci::server: bound tcp listener local_addr=127.0.0.1:26658
```

### Run Tendermint

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

#### Start Tendermint

Now we are ready to start Tendermint and let it connect to the Fendermint Application.

```shell
tendermint start
```
