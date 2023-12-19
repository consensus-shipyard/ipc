# IPC Monorepo

This is a monorepo to simplify the synchronisation effort that used to plague development of features that span the following constituent repositories:
* https://github.com/consensus-shipyard/fvm-utils
* https://github.com/consensus-shipyard/ipc-solidity-actors
* https://github.com/consensus-shipyard/ipc-ipld-resolver
* https://github.com/consensus-shipyard/ipc
* https://github.com/consensus-shipyard/fendermint

These repositories were addes as subtrees, which should allow us to pull in more updates from them while they are still alive.

The original setup command were as follows, all consolidating some local checkouts:

```shell
git subtree add -P contracts ../ipc-solidity-actors dev
git subtree add -P fendermint ../fendermint main
git subtree add -P ipc ../ipc dev
git subtree add -P fvm-utils ../fvm-utils main
git subtree add -P ipld-resolver ../ipc-ipld-resolver main
```

You may have to run `git submodule update --init --recursive` to initialize all the submodules under `contracts`.

TODO: Add examples of pulling updates from the upstream repos.
