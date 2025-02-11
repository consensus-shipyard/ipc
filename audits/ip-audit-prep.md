# IPC Audit Prep

# Overview

| Project Name | Hoku |
| --- | --- |
| Repositories | [https://github.com/hokunet/ipc/tree/main/contracts](https://github.com/hokunet/ipc/tree/main/contracts) |
| Commit | 786b6fc6678ff30addcd033b85f55dfd107e4e1c |
| Language | Solidity |
| Scope | All Solidity contracts in [https://github.com/hokunet/ipc/tree/main/contracts](https://github.com/hokunet/ipc/tree/main/contracts) |

## Process

- **Static Analysis:**  Auditor ran [Slither](https://github.com/crytic/slither) on the codebase to identify common vulnerabilities
- **Manual Code Review:**  Auditor ****manually reviewed the code to identify areas not following best practices and to catch potential vulnerabilities

# Findings

## Medium

### M1:  Account’s genesis block balance not set to zero correctly

[https://github.com/hokunet/ipc/blob/main/contracts/contracts/subnet/SubnetActorManagerFacet.sol#L258](https://github.com/hokunet/ipc/blob/main/contracts/contracts/subnet/SubnetActorManagerFacet.sol#L258)

The `msg.sender`'s genesis block balance is not correctly set to 0 when they leave as it currently does `s.genesisBalance[msg.sender] == 0;`   instead of `s.genesisBalance[msg.sender] = 0;`

**Recommendation:**  Set the `msg.sender`'s genesis balance to zero by either explicitly setting it to 0 **OR** deleting it’s storage using `delete s.genesisBalance[msg.sender]`

**Resolution:** The team has fixed this [here](https://github.com/consensus-shipyard/ipc/pull/1254/files#diff-73759bec6a63a717a540d6c2de7fe08b39c0a40532561115bdd91e824a5e6374L258).

## Low

### **L1:  Missing subnet validation for cross message recipient**

[https://github.com/hokunet/ipc/blob/main/contracts/contracts/gateway/GatewayMessengerFacet.sol#L59](https://github.com/hokunet/ipc/blob/main/contracts/contracts/gateway/GatewayMessengerFacet.sol#L59)

There is no validation for the subnet in `envelope.to`.  This means that it is currently possible to commit cross messages that are targeted to an invalid subnet.

**Recommendation:**  Validate that the subnet in [`envelope.to`](http://envelope.to) is registered in `sendContractXnetMessage`.

**Resolution:** The team has acknowledged this and have elected not to address it as the subnet does not know what the destination is on the destination chain.

### L2:  Incorrect comparison when checking minimum number of validators

[https://github.com/hokunet/ipc/blob/main/contracts/contracts/lib/LibSubnetActor.sol#L123](https://github.com/hokunet/ipc/blob/main/contracts/contracts/lib/LibSubnetActor.sol#L123)

The check should be updated to

```solidity
  uint256 length = validators.length;
	
	// Was previously  if (length <= s.minValidators)
  if (length < s.minValidators) {
      revert NotEnoughGenesisValidators();
  }
```

The current check will revert if the minimum number of validators are provided

**Recommendation:**  Update check to allow having the minimum number of validators

**Resolution:**  The team has fixed this [here](https://github.com/consensus-shipyard/ipc/pull/1254/files#diff-1f3ddd42139663a5b3f5b4092808a0b57c39f19f50fb41829194067fd0320791R123)

### L3:  Missing Gas Limit Validation when sending Cross Messages

`sendContractXnetMessage`

[https://github.com/hokunet/ipc/blob/main/contracts/contracts/gateway/GatewayMessengerFacet.sol#L33](https://github.com/hokunet/ipc/blob/main/contracts/contracts/gateway/GatewayMessengerFacet.sol#L33)

`applyCrossMessages`

[https://github.com/hokunet/ipc/blob/main/contracts/contracts/gateway/router/XnetMessagingFacet.sol#L29](https://github.com/hokunet/ipc/blob/main/contracts/contracts/gateway/router/XnetMessagingFacet.sol#L29)

`executeCrossMsg`

[https://github.com/hokunet/ipc/blob/main/contracts/contracts/lib/LibGateway.sol#L426](https://github.com/hokunet/ipc/blob/main/contracts/contracts/lib/LibGateway.sol#L426)

The `sendContractXnetMessage` function allows any address to send a cross message from one subnet to a call function on a target address on another subnet.  This message is eventually executed by calling `applyCrossMessages` on the target subnet’s Gateway, which calls `executeCrossMsg` internally.  Inside `executeCrossMsg` , the message is forwarded to the target address using `delegatecall` without putting any limits on the amount of gas consumed.  This allows a malicious actor to craft a malicious cross message that calls a gas intensive function on the target using `sendContractXnetMessage`.  The consequence of this is that `applyCrossMessages` might be unable to successfully process any messages as all of the gas is consumed by the malicious cross message.

**Recommendation:**  Limit the amount of gas consumed when calling the target address in `executeCrossMsg`.

```solidity
(success, result) = address(CrossMsgHelper).delegatecall{gasLimit: s.gasLimit}(   // solhint-disable-line avoid-low-level-calls
      abi.encodeWithSelector(CrossMsgHelper.execute.selector, crossMsg, supplySource)
  );
```

**Resolution:**  The team has acknowledged this and have elected to tackle this in the future.

## Informational

### I1:  Follow CEI (Checks Effects Interaction) Pattern in `GatewayManagerFacet`

[https://github.com/hokunet/ipc/blob/main/contracts/contracts/gateway/GatewayManagerFacet.sol#L33](https://github.com/hokunet/ipc/blob/main/contracts/contracts/gateway/GatewayManagerFacet.sol#L33)

The `register` function in `GatewayManagerFacet` is called by the `SubwayActorDiamond` contract in order to register itself.  Currently the function locks the supply and collateral tokens (Interaction) before validating (Check) the subnet and updating it’s storage state (Effects).  It is generally recommended to follow the Checks Effects Interaction pattern in order to guard against any possible reentrancy.  This finding has been marked as informational as there does not seem to be any malicious outcomes if the reentrancy is exploited.

**Recommendation:**  Rewrite the `register` function to be

```solidity
  function register(uint256 genesisCircSupply, uint256 collateral) external payable {
        // 1.  Perform Checks
        if (s.networkName.route.length + 1 >= s.maxTreeDepth) {
            revert MethodNotAllowed(ERR_CHILD_SUBNET_NOT_ALLOWED);
        }
        
        SubnetID memory subnetId = s.networkName.createSubnetId(msg.sender);

        (bool registered, Subnet storage subnet) = LibGateway.getSubnet(subnetId);
        if (registered) {
            revert AlreadyRegisteredSubnet();
        }
        
        // 2.  Perform Effects
        
         subnet.id = subnetId;
        subnet.stake = collateral;
        subnet.genesisEpoch = block.number;
        subnet.circSupply = genesisCircSupply;

        s.subnetKeys.add(subnetId.toHash());
        s.totalSubnets += 1;
        
        // 3. Perform Interactions

        if (genesisCircSupply > 0) {
            SubnetActorGetterFacet(msg.sender).supplySource().lock(genesisCircSupply);
        }
        if (collateral > 0) {
            SubnetActorGetterFacet(msg.sender).collateralSource().lock(collateral);
        }
    }
```

**Resolution:**  The team has fixed this [here](https://github.com/consensus-shipyard/ipc/pull/1254/files#diff-af0bf141fbc4fa3a8197c97b198e06bd01f09e44c83cbbe9f631d1e779768c8dR56-R62).

### I2:  Follow CEI Pattern in `addStake` function in `GatewayManagerFacet`

[https://github.com/hokunet/ipc/blob/main/contracts/contracts/gateway/GatewayManagerFacet.sol#L64](https://github.com/hokunet/ipc/blob/main/contracts/contracts/gateway/GatewayManagerFacet.sol#L64)

Similar to I1, it is recommended to follow CEI in `addStake`

**Recommendation:**  Update `addStake` 

```solidity
 function addStake(uint256 amount) external payable {
        if (amount == 0) {
            revert NotEnoughFunds();
        }
        
        (bool registered, Subnet storage subnet) = LibGateway.getSubnet(msg.sender);

        if (!registered) {
            revert NotRegisteredSubnet();
        }
        
        subnet.stake += amount;

        // The fund flow for stake is from Validator -> SubnetActor -> Gateway.
        // Because msg.sender is actually the subnet actor, this method sends the fund from
        // the subnet actor caller the gateway.
        SubnetActorGetterFacet(msg.sender).collateralSource().lock(amount);

    }
```

**Resolution:**  The team has acknowledged this 

### I3:  Complex EOA check can be simplified

[https://github.com/hokunet/ipc/blob/main/contracts/contracts/gateway/GatewayMessengerFacet.sol#L41](https://github.com/hokunet/ipc/blob/main/contracts/contracts/gateway/GatewayMessengerFacet.sol#L41)

The check for whether or not `msg.sender` is an EOA can be simplified to

```solidity
 if (msg.sender.code.length == 0) {
    revert InvalidXnetMessage(InvalidXnetMessageReason.Sender);
 }
```

**Resolution:**  The team has fixed this [here](https://github.com/consensus-shipyard/ipc/pull/1254/files#diff-71559bfe927afaf1a3980c01ffe9f4d15c77d58650b2d850bbae821de9c009d0R48).

### I4:  Transfer limits in cross messages

[https://github.com/hokunet/ipc/blob/main/contracts/contracts/gateway/GatewayMessengerFacet.sol#L60](https://github.com/hokunet/ipc/blob/main/contracts/contracts/gateway/GatewayMessengerFacet.sol#L60)

There is currently no limit to the amount of assets that can be sent in cross messages.  Consider adding some limits to the amount of assets that can be transferred by a sender within some timeframe to gradually monitor the behavior of the system before increasing the limits. 

**Resolution:**  The team has acknowledged this

### I5:  Missing NatSpec comments

The following functions are missing NatSpec comments

`setValidatorGater` is missing NatSpec comments describing the function and it’s parameter

[https://github.com/hokunet/ipc/blob/main/contracts/contracts/subnet/SubnetActorManagerFacet.sol#L76](https://github.com/hokunet/ipc/blob/main/contracts/contracts/subnet/SubnetActorManagerFacet.sol#L76)

**Recommendation:**  Add NatSpec comments to describe the purpose of a function, it’s parameters and it’s return types.

**Resolution:**  The team has addressed this [here](https://github.com/consensus-shipyard/ipc/pull/1254/files#diff-73759bec6a63a717a540d6c2de7fe08b39c0a40532561115bdd91e824a5e6374R79-R80).

### I6:  **Missing Event Emissions**

Consider emitting an event in the following functions

`SubnetActorManagerFacet` `setValidatorGater`

[https://github.com/hokunet/ipc/blob/main/contracts/contracts/subnet/SubnetActorManagerFacet.sol#L76](https://github.com/hokunet/ipc/blob/main/contracts/contracts/subnet/SubnetActorManagerFacet.sol#L76)

`SubnetActorManagerFacet` `kill`

[https://github.com/hokunet/ipc/blob/main/contracts/contracts/subnet/SubnetActorManagerFacet.sol#L274](https://github.com/hokunet/ipc/blob/main/contracts/contracts/subnet/SubnetActorManagerFacet.sol#L274)

`SubentActorManagerFacet` `addBootstrapNode`

[https://github.com/hokunet/ipc/blob/main/contracts/contracts/subnet/SubnetActorManagerFacet.sol#L287](https://github.com/hokunet/ipc/blob/main/contracts/contracts/subnet/SubnetActorManagerFacet.sol#L287)

**Recommendation:**  Emit events in state changing functions so that off-chain indexers can index them if needed.

**Resolution:**  The team has partially addressed this [here](https://github.com/consensus-shipyard/ipc/pull/1254/files#diff-73759bec6a63a717a540d6c2de7fe08b39c0a40532561115bdd91e824a5e6374R84) and [here](https://github.com/consensus-shipyard/ipc/pull/1254/files#diff-73759bec6a63a717a540d6c2de7fe08b39c0a40532561115bdd91e824a5e6374R306) 

### I7:  Delete unused memory from storage

Memory that is no longer used can be deleted to get some gas refunds.

[https://github.com/hokunet/ipc/blob/main/contracts/contracts/subnet/SubnetActorManagerFacet.sol#L258](https://github.com/hokunet/ipc/blob/main/contracts/contracts/subnet/SubnetActorManagerFacet.sol#L258)

**Resolution:** The team has addressed this [here](https://github.com/consensus-shipyard/ipc/pull/1254/files#diff-73759bec6a63a717a540d6c2de7fe08b39c0a40532561115bdd91e824a5e6374L258).

### I8:  Inefficient incrementing of storage variables

Variables are incremented using `+= 1` in several places in the code.  It is generally more gas efficient to pre-increment variables.

[https://github.com/hokunet/ipc/blob/main/contracts/contracts/lib/LibGateway.sol#L259](https://github.com/hokunet/ipc/blob/main/contracts/contracts/lib/LibGateway.sol#L259)

can be updated to `++subnet.appliedBottomUpNonce;`

[https://github.com/hokunet/ipc/blob/main/contracts/contracts/lib/LibGateway.sol#L384](https://github.com/hokunet/ipc/blob/main/contracts/contracts/lib/LibGateway.sol#L384)

can be updated to `++s.appliedTopDownNonce;`

**Recommendation:**  Pre-increment variables to save gas

**Resolution:** The team has acknowledged this