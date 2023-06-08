# Child Subnet Infra 
This is a utility script to spawn child subnet infrastructure, mainly for integration testing. 

It spawns the required number of eudico nodes and validators based on input config parameters. This would be a time saver 
when testing subnet checkpoints and cross net messages.

The `src/infra` contains the process flow for spawn subnet nodes and their validators.
The `examples` folder contains all the sample scripts to start different configurations and requirements.

To test this, one must first manually start the root net and the ipc agent. Then start the process with:
```shell
cargo build --release

# start the root subnet and ipc agent on your own

# define the following env variable
# the path to eudico binary path
export EUDICO_BIN=
# the ipc root folder that contains all the configs, i.e. ~/.ipc-agent
export IPC_ROOT_FOLDER=
# the parent subnet lotus path
export PARENT_LOTUS_PATH=
# the parent subnet id
export PARENT_SUBNET_ID=
# the name of the subnet
export SUBNET_NAME=

# start the infra that runs util kill signal
../../target/release/examples/no_tear_down

# in another terminal, define the following variables
# the address that performs the funding and release across subnets 
export FUND_ADDRESS=
# the ipc agent json rpc url 
export IPC_AGENT_JSON_RPC_URL=http://localhost:3030/json_rpc
# child subnet id in string format
export CHILD_SUBNET_ID_STR=
 
# start the tests with 
cargo test -p itest --test checkpoint -- --nocapture
```