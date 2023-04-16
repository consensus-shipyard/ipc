# Using the IPC Agent

## Listing active subnets

As a sanity-check that we have joined the subnet successfully and that we provided enough collateral to register the subnet to IPC, we can list the child subnets of our parent with the following command:

```bash
$ ./bin/ipc-agent list-subnets --gateway-address=<gateway-addr> --subnet=<parent-subnet-id>

# Sample execution
$ ./bin/ipc-agent list-subnets --gateway-address=t064 --subnet=/root
[2023-03-30T17:00:25Z INFO  ipc_agent::cli::commands::manager::list_subnets] /root/t01003 - status: 0, collateral: 2 FIL, circ.supply: 0.0 FIL
```

This command only shows subnets that have been registered to the gateway, i.e. that have provided enough collateral to participate in the IPC protocol and haven't been killed. It is not an exhaustive list of all of the subnet actors deployed over the network.

## Joining a subnet

With the daemon for a subnet deployed (see [instructions](/docs/subnet.md)), one can join the subnet:
```bash
$ ./bin/ipc-agent subnet join --subnet <subnet-id> --collateral <collateral_amount> --validator-net-addr <libp2p-add-validator>

# Example execution
$ ./bin/ipc-agent subnet join --subnet /root/t01002 --collateral 2 --validator-net-addr /dns/host.docker.internal/tcp/1349/p2p/12D3KooWN5hbWkCxwvrX9xYxMwFbWm2Jpa1o4qhwifmSw3Fb
```
This command specifies the subnet to join, the amount of collateral to provide and the validator net address used by other validators to dial them.

## Sending funds in a subnet

The agent provides a command to conveniently exchange funds between addresses of the same subnet. This can be achieved through the following command:
```bash
$ ./bin/ipc-agent subnet send-value --subnet <subnet-id> --to <to-addr> <value>

# Example execution
$ ./bin/ipc-agent subnet send-value --subnet /root/t01002 --to t1xbevqterae2tanmh2kaqksnoacflrv6w2dflq4i 10
```

## Leaving a subnet

To leave a subnet, the following agent command can be used:
```bash
$ ./bin/ipc-agent subnet leave --subnet <subnet-id>

# Example execution
$ ./bin/ipc-agent subnet leave --subnet /root/t01002
```
Leaving a subnet will release the collateral for the validator and remove all the validation rights from its account. This means that if you have a validator running in that subnet, its validation process will immediately terminate.


### Listing checkpoints from a subnet

Subnets are periodically committing checkpoints to their parent every `check-period` (parameter defined when creating the subnet). If you want to inspect the information of a range of checkpoints committed in the parent for a subnet, you can use the `checkpoint list` command provided by the agent as follows: 
```bash
# List checkpoints between two epochs for a subnet
$ ./bin/ipc-agent checkpoint list --from-epoch <range-start> --to-epoch <range-end> --subnet <subnet-id>

# Example execution
$ ./bin/ipc-agent checkpoint list --from-epoch 0 --to-epoch 100 --subnet root/t01002
[2023-03-29T12:43:42Z INFO  ipc_agent::cli::commands::manager::list_checkpoints] epoch 0 - prev_check={"/":"bafy2bzacedkoa623kvi5gfis2yks7xxjl73vg7xwbojz4tpq63dd5jpfz757i"}, cross_msgs=null, child_checks=null
[2023-03-29T12:43:42Z INFO  ipc_agent::cli::commands::manager::list_checkpoints] epoch 10 - prev_check={"/":"bafy2bzacecsatvda6lodrorh7y7foxjt3a2dexxx5jiyvtl7gimrrvywb7l5m"}, cross_msgs=null, child_checks=null
[2023-03-29T12:43:42Z INFO  ipc_agent::cli::commands::manager::list_checkpoints] epoch 30 - prev_check={"/":"bafy2bzaceauzdx22hna4e4cqf55jqmd64a4fx72sxprzj72qhrwuxhdl7zexu"}, cross_msgs=null, child_checks=null
```
