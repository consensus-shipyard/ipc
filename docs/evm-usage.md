# IPC Agent EVM Support

## Key management
The IPC agent has internally and EVM wallet that it uses to sign transactions and interact with IPC on behalf of specific addresses. Some of the features available for EVM addresses through the EVM are: 
* Creating new Ethereum addresses
```bash
./bin/ipc-agent wallet new -w evm

# Sample execution
./bin/ipc-agent wallet new -w evm

[2023-07-12T10:58:37Z INFO  ipc_agent::cli::commands::wallet::new] created new wallet with address WalletNewResponse { address: "0x6972968294f71daba16939f77dfd738525796a63" }
```

* Exporting a key stored in the IPC agent. 
```bash
./bin/ipc-agent wallet export -w evm -a <EVM-ADDRESS> -o <OUTPUT_FILE>

# Sample execution
./bin/ipc-agent wallet export -w evm -a 0x92e2dd319dae2f5698ef5cbde610ad611983de0d -o ~/.ipc-agent/evm-wallet.key

[2023-07-12T11:00:01Z INFO  ipc_agent::cli::commands::wallet::export] exported new wallet with address "0x92e2dd319dae2f5698ef5cbde610ad611983de0d" in file "~/.ipc-agent/evm-wallet.key"
```

* Importing a key from a file
```bash
./bin/ipc-agent wallet import -w evm --path=<INPUT_FILE_WITH_KEY>

# Sample execution
./bin/ipc-agent wallet import -w evm --path=~/.ipc-agent/evm-wallet.key
[2023-07-12T11:00:59Z INFO  ipc_agent::cli::commands::wallet::import] imported wallet with address "0x92e2…de0d"
```

> 💡 The format expected to import new EVM keys is the following:
> ```
> {"address":<EVM-ADDRESS>,"private_key":<PRIVATE_KEY>}
> ```
> You can always create this file manually to import some address into the agent that you have exported from some other tool with an alternative format.

* Importing an identity directly from its private key
```bash
./bin/ipc-agent wallet import -w evm --private-key <PRIVATE_KEY>

# Sample execution
./bin/ipc-agent wallet import -w evm --private-key=0x405f50458008edd6e2eb2efc3bf34846db1d6689b89fe1a9f9ccfe7f6e301d8d

[2023-07-12T11:00:59Z INFO  ipc_agent::cli::commands::wallet::import] imported wallet with address "0x92e2…de0d"
