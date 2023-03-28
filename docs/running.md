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
  cargo run -p fendermint_app -- keygen --out-dir test-network/keys --name $NAME;
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
cat test-network/genesis.json | jq .accounts[1]
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

Finally, let's add all validators with equal power to Genesis:

```shell
for NAME in alice bob charlie dave; do
  cargo run -p fendermint_app -- \
        genesis --genesis-file test-network/genesis.json \
        add-validator --public-key test-network/keys/$NAME.pk --power 25;
done
```

Check that all of them are present:

```console
❯ cat test-network/genesis.json | jq .validators
[
  {
    "public_key": "BE5Juk793ZAg/7Ojj4bzOmIFGpwLhET1vg2ROihUJFkqGC63X6tOBnky31kw7wPqL0tvbPrtLM2O+SooUhiV1Mo=",
    "power": 25
  },
  {
    "public_key": "BCImfwVC/LeFJN9bB612aCtjbCYWuilf2SorSUXez/QEy8cVKNuvTU/EOTibo3hIyOQslvSouzIpR24h1kkqCSI=",
    "power": 25
  },
  {
    "public_key": "BJVVXUBEjwW8DyZIXb2iw7aq6DJF14kdcCYqKdyruQJAOMGlBR5jSGgeM8O+BX+E2+etsm2xIoWAQllZtY4K9is=",
    "power": 25
  },
  {
    "public_key": "BPcq6nnj38i6fhK7GlRVPLE870QJD88ZwalM3ySDadBAHXSlD5AYAd7JZFjYnDf4WtwEcDfodIuiXchRw9389bM=",
    "power": 25
  }
]
```

The public keys are spliced in as they were, in base64 format, which is how they would appear in Tendermint's
own genesis file format. Note that here we don't have the option to use `Address`, because we have to return
these as actual `PublicKey` types to Tendermint through ABCI, not as a hash of a key.
