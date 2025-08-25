# IPC Contract Errors Reference

This document provides a comprehensive reference for all possible errors that can occur when interacting with IPC (InterPlanetary Consensus) smart contracts. Understanding these errors will help developers and users diagnose issues and understand what went wrong.

## Table of Contents

-   [Authentication & Authorization Errors](#authentication--authorization-errors)
-   [Subnet Management Errors](#subnet-management-errors)
-   [Validator Management Errors](#validator-management-errors)
-   [Checkpoint & Batch Errors](#checkpoint--batch-errors)
-   [Cross-Message Errors](#cross-message-errors)
-   [Financial & Balance Errors](#financial--balance-errors)
-   [Configuration Errors](#configuration-errors)
-   [Signature & Security Errors](#signature--security-errors)
-   [Diamond Pattern Errors](#diamond-pattern-errors)
-   [General System Errors](#general-system-errors)

---

## Authentication & Authorization Errors

### `NotOwner()`

**When it occurs:** When a function that requires ownership is called by an address that is not the contract owner.
**What it means:** Only the contract owner can perform this operation.
**How to fix:** Ensure you're calling the function from the correct owner address.

### `NotAuthorized(address)`

**When it occurs:** When a function is called by an address that doesn't have the required permissions.
**What it means:** The specified address lacks the necessary authorization for this operation.
**How to fix:** Verify that the calling address has the required role or permissions.

### `NotOwnerOfPublicKey()`

**When it occurs:** When trying to perform an operation that requires ownership of a specific public key.
**What it means:** The caller doesn't own the public key being referenced.
**How to fix:** Ensure you're using the correct key pair or obtain ownership of the required public key.

### `NotSystemActor()`

**When it occurs:** When a non-system actor tries to perform a system-level operation.
**What it means:** Only system actors (like the gateway) can perform this operation.
**How to fix:** This operation should only be called by authorized system contracts.

---

## Subnet Management Errors

### `AlreadyRegisteredSubnet()`

**When it occurs:** When trying to register a subnet that is already registered.
**What it means:** The subnet ID is already in use.
**How to fix:** Use a different subnet ID or check if the subnet already exists.

### `NotRegisteredSubnet()`

**When it occurs:** When trying to perform operations on a subnet that doesn't exist.
**What it means:** The subnet hasn't been registered yet.
**How to fix:** Register the subnet first before performing operations on it.

### `SubnetNotFound()`

**When it occurs:** When trying to access a subnet that doesn't exist in the registry.
**What it means:** The subnet ID doesn't correspond to any registered subnet.
**How to fix:** Verify the subnet ID or register the subnet first.

### `SubnetNotActive()`

**When it occurs:** When trying to perform operations on a subnet that is not in an active state.
**What it means:** The subnet may be paused, killed, or in an inactive state.
**How to fix:** Ensure the subnet is active before performing operations.

### `SubnetAlreadyKilled()`

**When it occurs:** When trying to kill a subnet that is already killed.
**What it means:** The subnet has already been terminated.
**How to fix:** Check the subnet status before attempting to kill it.

### `SubnetAlreadyBootstrapped()`

**When it occurs:** When trying to bootstrap a subnet that is already bootstrapped.
**What it means:** The subnet has already completed the bootstrap process.
**How to fix:** Check if the subnet is already bootstrapped before attempting to bootstrap it.

### `SubnetNotBootstrapped()`

**When it occurs:** When trying to perform operations that require a bootstrapped subnet.
**What it means:** The subnet hasn't completed the bootstrap process yet.
**How to fix:** Complete the subnet bootstrap process first.

### `UnknownSubnet()`

**When it occurs:** When trying to interact with a subnet that is not recognized.
**What it means:** The subnet ID is not valid or not recognized by the system.
**How to fix:** Verify the subnet ID and ensure it's properly registered.

### `CannotFindSubnet()`

**When it occurs:** When the system cannot locate a subnet with the given parameters.
**What it means:** The subnet lookup failed due to invalid parameters or missing data.
**How to fix:** Check the subnet parameters and ensure the subnet exists.

---

## Validator Management Errors

### `AddressShouldBeValidator()`

**When it occurs:** When an address that should be a validator is not recognized as one.
**What it means:** The address is not registered as a validator in the subnet.
**How to fix:** Ensure the address is properly registered as a validator.

### `NotValidator(address)`

**When it occurs:** When trying to perform validator-specific operations with a non-validator address.
**What it means:** The specified address is not a validator in the subnet.
**How to fix:** Register the address as a validator or use a validator address.

### `NoValidatorsInSubnet()`

**When it occurs:** When trying to perform operations that require validators but none exist.
**What it means:** The subnet has no registered validators.
**How to fix:** Register validators in the subnet before performing validator-dependent operations.

### `NotEnoughValidatorsInSubnet()`

**When it occurs:** When there are insufficient validators for an operation.
**What it means:** The minimum required number of validators is not met.
**How to fix:** Add more validators to meet the minimum requirement.

### `NotEnoughGenesisValidators()`

**When it occurs:** When trying to bootstrap a subnet without enough genesis validators.
**What it means:** The minimum number of genesis validators required for bootstrap is not met.
**How to fix:** Add more genesis validators before bootstrapping.

### `ValidatorAlreadyClaimed()`

**When it occurs:** When trying to claim rewards that have already been claimed.
**What it means:** The validator has already claimed their rewards for this period.
**How to fix:** Wait for the next reward period or check if rewards are already claimed.

### `ValidatorPowerChangeDenied()`

**When it occurs:** When a validator power change is rejected by the system.
**What it means:** The power change violates system rules or constraints.
**How to fix:** Ensure the power change complies with system requirements.

### `NotAllValidatorsHaveLeft()`

**When it occurs:** When trying to perform an operation that requires all validators to have left.
**What it means:** Some validators are still active in the subnet.
**How to fix:** Wait for all validators to leave or remove them manually.

### `DuplicatedGenesisValidator()`

**When it occurs:** When trying to add a genesis validator that already exists.
**What it means:** The validator is already registered as a genesis validator.
**How to fix:** Check if the validator is already registered before adding.

---

## Checkpoint & Batch Errors

### `CheckpointAlreadyExists()`

**When it occurs:** When trying to create a checkpoint that already exists.
**What it means:** A checkpoint for this epoch already exists.
**How to fix:** Check if the checkpoint already exists before creating it.

### `CheckpointNotCreated()`

**When it occurs:** When trying to perform operations on a checkpoint that doesn't exist.
**What it means:** The checkpoint hasn't been created yet.
**How to fix:** Create the checkpoint first before performing operations on it.

### `BottomUpCheckpointAlreadySubmitted()`

**When it occurs:** When trying to submit a bottom-up checkpoint that's already been submitted.
**What it means:** The checkpoint has already been submitted to the parent chain.
**How to fix:** Check if the checkpoint is already submitted before attempting to submit it.

### `InvalidCheckpointEpoch()`

**When it occurs:** When trying to create or access a checkpoint with an invalid epoch.
**What it means:** The epoch number is not valid or not in the expected range.
**How to fix:** Use a valid epoch number within the acceptable range.

### `InvalidCheckpointSource()`

**When it occurs:** When the checkpoint source is not valid.
**What it means:** The checkpoint is coming from an unauthorized or invalid source.
**How to fix:** Ensure the checkpoint is from an authorized source.

### `DuplicatedCheckpointHeight(uint64 height)`

**When it occurs:** When trying to create a checkpoint with a height that already exists.
**What it means:** A checkpoint with this height has already been created.
**How to fix:** Use a different height or check if the checkpoint already exists.

### `BatchAlreadyExists()`

**When it occurs:** When trying to create a batch that already exists.
**What it means:** A batch for this epoch already exists.
**How to fix:** Check if the batch already exists before creating it.

### `BatchNotCreated()`

**When it occurs:** When trying to perform operations on a batch that doesn't exist.
**What it means:** The batch hasn't been created yet.
**How to fix:** Create the batch first before performing operations on it.

### `BatchMsgAlreadyExecuted()`

**When it occurs:** When trying to execute a batch message that's already been executed.
**What it means:** The message has already been processed.
**How to fix:** Check if the message is already executed before attempting to execute it.

### `InvalidBatchEpoch()`

**When it occurs:** When trying to create or access a batch with an invalid epoch.
**What it means:** The epoch number is not valid for batch operations.
**How to fix:** Use a valid epoch number for batch operations.

### `InvalidBatchSource()`

**When it occurs:** When the batch source is not valid.
**What it means:** The batch is coming from an unauthorized or invalid source.
**How to fix:** Ensure the batch is from an authorized source.

### `MissingBatchCommitment()`

**When it occurs:** When a batch commitment is missing for an operation.
**What it means:** The required batch commitment data is not available.
**How to fix:** Ensure the batch commitment is properly generated and available.

### `MissingActivityCommitment()`

**When it occurs:** When an activity commitment is missing for an operation.
**What it means:** The required activity commitment data is not available.
**How to fix:** Ensure the activity commitment is properly generated and available.

### `QuorumAlreadyProcessed()`

**When it occurs:** When trying to process a quorum that's already been processed.
**What it means:** The quorum has already been handled by the system.
**How to fix:** Check if the quorum is already processed before attempting to process it.

### `FailedAddIncompleteQuorum()`

**When it occurs:** When trying to add an incomplete quorum to the system.
**What it means:** The quorum doesn't meet the completeness requirements.
**How to fix:** Ensure the quorum is complete before adding it.

### `FailedAddSignatory()`

**When it occurs:** When trying to add a signatory fails.
**What it means:** The signatory addition operation failed due to validation or system constraints.
**How to fix:** Check the signatory data and ensure it meets requirements.

### `FailedRemoveIncompleteQuorum()`

**When it occurs:** When trying to remove an incomplete quorum from the system.
**What it means:** The quorum removal operation failed due to system constraints.
**How to fix:** Ensure the quorum can be safely removed.

---

## Cross-Message Errors

### `InvalidXnetMessage(InvalidXnetMessageReason reason)`

**When it occurs:** When a cross-network message is invalid.
**What it means:** The message violates cross-network communication rules.
**How to fix:** Check the specific reason and ensure the message complies with cross-network rules.

**Possible reasons:**

-   `Sender`: Invalid sender address
-   `DstSubnet`: Invalid destination subnet
-   `Nonce`: Invalid nonce value
-   `Value`: Invalid message value
-   `Kind`: Invalid message kind
-   `ReflexiveSend`: Attempting to send to the same subnet
-   `NoRoute`: No route available to destination
-   `IncompatibleSupplySource`: Incompatible supply source

### `CannotExecuteEmptyEnvelope()`

**When it occurs:** When trying to execute an empty cross-message envelope.
**What it means:** The message envelope contains no executable content.
**How to fix:** Ensure the message envelope contains valid content before execution.

### `PostboxNotExist()`

**When it occurs:** When trying to access a postbox that doesn't exist.
**What it means:** The postbox for cross-message communication is not available.
**How to fix:** Ensure the postbox is properly initialized.

---

## Financial & Balance Errors

### `NotEnoughBalance()`

**When it occurs:** When trying to perform an operation that requires more balance than available.
**What it means:** The account doesn't have sufficient balance for the operation.
**How to fix:** Ensure sufficient balance before performing the operation.

### `NotEnoughFunds()`

**When it occurs:** When trying to perform an operation that requires more funds than available.
**What it means:** The contract or account doesn't have sufficient funds.
**How to fix:** Add more funds or reduce the operation cost.

### `NotEnoughFundsToRelease()`

**When it occurs:** When trying to release funds that exceed available balance.
**What it means:** The requested release amount is greater than available funds.
**How to fix:** Request a smaller amount or add more funds.

### `NotEnoughCollateral()`

**When it occurs:** When trying to perform an operation that requires more collateral than available.
**What it means:** Insufficient collateral is staked for the operation.
**How to fix:** Stake more collateral before performing the operation.

### `NotEnoughBalanceForRewards()`

**When it occurs:** When trying to distribute rewards but insufficient balance is available.
**What it means:** The reward pool doesn't have enough balance to distribute rewards.
**How to fix:** Add more funds to the reward pool or reduce reward amounts.

### `NotEnoughSubnetCircSupply()`

**When it occurs:** When trying to perform an operation that requires more subnet circulating supply than available.
**What it means:** The subnet doesn't have enough circulating supply for the operation.
**How to fix:** Ensure sufficient circulating supply in the subnet.

### `NotEmptySubnetCircSupply()`

**When it occurs:** When trying to perform an operation that requires empty circulating supply.
**What it means:** The subnet still has circulating supply when it should be empty.
**How to fix:** Wait for the circulating supply to be depleted or handle it appropriately.

### `NoCollateralToWithdraw()`

**When it occurs:** When trying to withdraw collateral that doesn't exist.
**What it means:** No collateral is available for withdrawal.
**How to fix:** Check if collateral is actually staked before attempting withdrawal.

### `CannotReleaseZero()`

**When it occurs:** When trying to release zero amount of funds.
**What it means:** The release operation requires a non-zero amount.
**How to fix:** Specify a non-zero amount for the release operation.

### `CollateralIsZero()`

**When it occurs:** When trying to perform operations with zero collateral.
**What it means:** The operation requires non-zero collateral.
**How to fix:** Provide non-zero collateral for the operation.

### `TransferFailed(address, address, uint256)`

**When it occurs:** When a token transfer operation fails.
**What it means:** The transfer from one address to another failed.
**How to fix:** Check token balances, allowances, and transfer conditions.

---

## Configuration Errors

### `InvalidConfigurationNumber()`

**When it occurs:** When trying to use an invalid configuration number.
**What it means:** The configuration number doesn't meet system requirements.
**How to fix:** Use a valid configuration number.

### `OldConfigurationNumber()`

**When it occurs:** When trying to use an outdated configuration number.
**What it means:** The configuration number is too old and no longer valid.
**How to fix:** Use a current configuration number.

### `InvalidMajorityPercentage()`

**When it occurs:** When trying to set an invalid majority percentage.
**What it means:** The majority percentage is outside the valid range.
**How to fix:** Use a majority percentage within the valid range (typically 50-100%).

### `InvalidPowerScale()`

**When it occurs:** When trying to set an invalid power scale.
**What it means:** The power scale value is outside the acceptable range.
**How to fix:** Use a power scale value within the valid range.

### `InvalidRetentionHeight()`

**When it occurs:** When trying to set an invalid retention height.
**What it means:** The retention height doesn't meet system requirements.
**How to fix:** Use a valid retention height value.

### `InvalidSubmissionPeriod()`

**When it occurs:** When trying to set an invalid submission period.
**What it means:** The submission period is outside the acceptable range.
**How to fix:** Use a submission period within the valid range.

### `InvalidActorAddress()`

**When it occurs:** When trying to use an invalid actor address.
**What it means:** The actor address doesn't meet validation requirements.
**How to fix:** Use a valid actor address.

### `GatewayCannotBeZero()`

**When it occurs:** When trying to set a zero address as gateway.
**What it means:** The gateway address cannot be the zero address.
**How to fix:** Provide a valid non-zero gateway address.

### `WrongGateway()`

**When it occurs:** When trying to use the wrong gateway for an operation.
**What it means:** The gateway address doesn't match the expected gateway.
**How to fix:** Use the correct gateway address for the operation.

### `FacetCannotBeZero()`

**When it occurs:** When trying to set a zero address as a diamond facet.
**What it means:** The facet address cannot be the zero address.
**How to fix:** Provide a valid non-zero facet address.

---

## Signature & Security Errors

### `InvalidSignature()`

**When it occurs:** When a signature validation fails.
**What it means:** The provided signature is not valid for the given data.
**How to fix:** Ensure the signature is correctly generated and matches the data.

### `InvalidSignatureErr(uint8)`

**When it occurs:** When a signature validation fails with a specific error code.
**What it means:** The signature validation failed with the specified error code.
**How to fix:** Check the error code and ensure proper signature generation.

### `InvalidSignatureLength()`

**When it occurs:** When a signature has an invalid length.
**What it means:** The signature length doesn't match the expected format.
**How to fix:** Ensure the signature has the correct length for the signature type.

### `InvalidPublicKeyLength()`

**When it occurs:** When a public key has an invalid length.
**What it means:** The public key length doesn't match the expected format.
**How to fix:** Ensure the public key has the correct length for the key type.

### `SignatureReplay()`

**When it occurs:** When trying to reuse a signature that's already been used.
**What it means:** The signature has already been consumed and cannot be reused.
**How to fix:** Generate a new signature for each operation.

### `SignatureAddressesNotSorted()`

**When it occurs:** When signature addresses are not in the required sorted order.
**What it means:** The addresses in a multi-signature are not properly sorted.
**How to fix:** Sort the addresses in ascending order before creating the signature.

### `DuplicateValidatorSignaturesFound()`

**When it occurs:** When duplicate validator signatures are found in a multi-signature.
**What it means:** The same validator has signed multiple times.
**How to fix:** Ensure each validator only signs once in a multi-signature.

### `NotOwnerOfPublicKey()`

**When it occurs:** When trying to use a public key that you don't own.
**What it means:** The caller doesn't have ownership of the specified public key.
**How to fix:** Use a public key that you own or obtain ownership.

---

## Diamond Pattern Errors

### `FunctionNotFound(bytes4 _functionSelector)`

**When it occurs:** When trying to call a function that doesn't exist in the diamond.
**What it means:** The function selector is not registered in the diamond contract.
**How to fix:** Check if the function exists or use a different function.

### `InvalidAddress()`

**When it occurs:** When trying to use an invalid address in diamond operations.
**What it means:** The address doesn't meet validation requirements.
**How to fix:** Use a valid address.

### `NoBytecodeAtAddress(address _contractAddress, string _message)`

**When it occurs:** When trying to add a facet that has no bytecode.
**What it means:** The contract address doesn't contain valid bytecode.
**How to fix:** Ensure the contract is properly deployed with bytecode.

### `IncorrectFacetCutAction(IDiamondCut.FacetCutAction _action)`

**When it occurs:** When using an incorrect facet cut action.
**What it means:** The action is not valid for the current operation.
**How to fix:** Use the correct facet cut action (Add, Replace, or Remove).

### `NoSelectorsProvidedForFacetForCut(address _facetAddress)`

**When it occurs:** When trying to cut a facet without providing function selectors.
**What it means:** No function selectors were specified for the facet cut operation.
**How to fix:** Provide the required function selectors for the facet cut.

### `CannotAddFunctionToDiamondThatAlreadyExists(bytes4 _selector)`

**When it occurs:** When trying to add a function that already exists in the diamond.
**What it means:** The function selector is already registered.
**How to fix:** Use a different function selector or replace the existing one.

### `CannotAddSelectorsToZeroAddress(bytes4[] _selectors)`

**When it occurs:** When trying to add selectors to a zero address facet.
**What it means:** Cannot add function selectors to the zero address.
**How to fix:** Provide a valid facet address.

### `InitializationFunctionReverted(address _initializationContractAddress, bytes _calldata)`

**When it occurs:** When the diamond initialization function reverts.
**What it means:** The initialization process failed.
**How to fix:** Check the initialization parameters and ensure the initialization contract is correct.

### `NoSelectorsGivenToAdd()`

**When it occurs:** When trying to add a facet without providing any selectors.
**What it means:** No function selectors were provided for the add operation.
**How to fix:** Provide at least one function selector for the add operation.

### `NotContractOwner(address _user, address _contractOwner)`

**When it occurs:** When a non-owner tries to perform diamond operations.
**What it means:** Only the contract owner can perform this operation.
**How to fix:** Ensure you're the contract owner or contact the owner.

### `CannotReplaceFunctionsFromFacetWithZeroAddress(bytes4[] _selectors)`

**When it occurs:** When trying to replace functions with a zero address facet.
**What it means:** Cannot replace functions with the zero address.
**How to fix:** Provide a valid facet address for replacement.

### `CannotReplaceImmutableFunction(bytes4 _selector)`

**When it occurs:** When trying to replace an immutable function.
**What it means:** The function is marked as immutable and cannot be replaced.
**How to fix:** Use a different function or remove the immutable constraint.

### `CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet(bytes4 _selector)`

**When it occurs:** When trying to replace a function with the same function from the same facet.
**What it means:** The replacement is identical to the current function.
**How to fix:** Provide a different function for replacement.

### `CannotReplaceFunctionThatDoesNotExists(bytes4 _selector)`

**When it occurs:** When trying to replace a function that doesn't exist.
**What it means:** The function selector is not registered in the diamond.
**How to fix:** Add the function first or use an existing function selector.

### `RemoveFacetAddressMustBeZeroAddress(address _facetAddress)`

**When it occurs:** When trying to remove a facet with a non-zero address.
**What it means:** Facet removal requires the zero address.
**How to fix:** Use the zero address for facet removal operations.

### `CannotRemoveFunctionThatDoesNotExist(bytes4 _selector)`

**When it occurs:** When trying to remove a function that doesn't exist.
**What it means:** The function selector is not registered in the diamond.
**How to fix:** Check if the function exists before attempting removal.

### `CannotRemoveImmutableFunction(bytes4 _selector)`

**When it occurs:** When trying to remove an immutable function.
**What it means:** The function is marked as immutable and cannot be removed.
**How to fix:** Use a different function or remove the immutable constraint.

---

## General System Errors

### `EmptyAddress()`

**When it occurs:** When trying to use an empty address where a valid address is required.
**What it means:** The address is empty or not properly set.
**How to fix:** Provide a valid non-empty address.

### `AlreadyInSet()`

**When it occurs:** When trying to add an item that's already in a set.
**What it means:** The item is already present in the collection.
**How to fix:** Check if the item exists before adding it.

### `NotInSet()`

**When it occurs:** When trying to perform operations on an item that's not in a set.
**What it means:** The item is not present in the collection.
**How to fix:** Add the item to the set first or check if it exists.

### `PQEmpty()`

**When it occurs:** When trying to perform operations on an empty priority queue.
**What it means:** The priority queue has no elements.
**How to fix:** Add elements to the priority queue before performing operations.

### `PQDoesNotContainAddress()`

**When it occurs:** When trying to perform operations on an address that's not in the priority queue.
**What it means:** The address is not present in the priority queue.
**How to fix:** Add the address to the priority queue first.

### `ZeroMembershipWeight()`

**When it occurs:** When trying to perform operations with zero membership weight.
**What it means:** The membership weight is zero when a non-zero value is required.
**How to fix:** Ensure the membership weight is greater than zero.

### `PowerReductionMoreThanTotal(uint256 total, uint256 change)`

**When it occurs:** When trying to reduce power by more than the total available power.
**What it means:** The power reduction amount exceeds the total power.
**How to fix:** Reduce the power reduction amount to be within the total power.

### `CannotConfirmFutureChanges()`

**When it occurs:** When trying to confirm changes that are in the future.
**What it means:** Cannot confirm changes that haven't been proposed yet.
**How to fix:** Wait for the changes to be proposed before confirming them.

### `ParentFinalityAlreadyCommitted()`

**When it occurs:** When trying to commit parent finality that's already committed.
**What it means:** The parent finality has already been committed.
**How to fix:** Check if the finality is already committed before attempting to commit it.

### `ReentrancyError()`

**When it occurs:** When a reentrancy attack is detected.
**What it means:** A function is trying to call itself or another function that could lead to reentrancy.
**How to fix:** Implement proper reentrancy guards or restructure the function calls.

### `NotDelegatedEvmAddress()`

**When it occurs:** When trying to use an EVM address that's not delegated.
**What it means:** The EVM address doesn't have the required delegation.
**How to fix:** Ensure the EVM address is properly delegated.

### `NoParentForSubnet()`

**When it occurs:** When trying to access a parent subnet that doesn't exist.
**What it means:** The subnet doesn't have a valid parent subnet.
**How to fix:** Ensure the subnet has a valid parent or check the subnet hierarchy.

### `NoAddressForRoot()`

**When it occurs:** When trying to get an address for a root subnet.
**What it means:** Root subnets don't have addresses in the same way as child subnets.
**How to fix:** Handle root subnets differently as they don't have parent addresses.

### `EmptySubnet()`

**When it occurs:** When trying to perform operations on an empty subnet.
**What it means:** The subnet has no content or is not properly initialized.
**How to fix:** Ensure the subnet is properly initialized with content.

### `MethodNotAllowed(string reason)`

**When it occurs:** When trying to call a method that's not allowed in the current state.
**What it means:** The method is restricted due to the current system state or configuration.
**How to fix:** Check the reason and ensure the system is in the correct state for the method.

### `InvalidFederationPayload()`

**When it occurs:** When the federation payload is invalid.
**What it means:** The payload doesn't meet federation requirements.
**How to fix:** Ensure the payload is properly formatted and valid.

### `InvalidActivityProof()`

**When it occurs:** When the activity proof is invalid.
**What it means:** The proof doesn't validate the claimed activity.
**How to fix:** Ensure the activity proof is correctly generated and valid.

### `MissingGenesisSubnetIpcContractsOwner()`

**When it occurs:** When the genesis subnet IPC contracts owner is missing.
**What it means:** The owner of the genesis subnet IPC contracts is not set.
**How to fix:** Set the owner for the genesis subnet IPC contracts.

---

## SDK-Specific Errors

### `CallerIsNotGateway()`

**When it occurs:** When a non-gateway contract tries to call gateway-specific functions.
**What it means:** Only gateway contracts can call this function.
**How to fix:** Ensure the caller is a valid gateway contract.

### `UnsupportedMsgKind()`

**When it occurs:** When trying to handle an unsupported message kind.
**What it means:** The message type is not supported by the current implementation.
**How to fix:** Use a supported message kind or update the implementation.

### `UnrecognizedResult()`

**When it occurs:** When the result of an operation is not recognized.
**What it means:** The operation returned an unexpected or unrecognized result.
**How to fix:** Check the operation parameters and ensure proper result handling.

---

## Pausable Errors

### `EnforcedPause()`

**When it occurs:** When trying to perform operations while the contract is paused.
**What it means:** The contract is in a paused state and operations are blocked.
**How to fix:** Wait for the contract to be unpaused or contact the contract owner.

### `ExpectedPause()`

**When it occurs:** When trying to pause a contract that's already paused.
**What it means:** The contract is already in a paused state.
**How to fix:** Check the current pause state before attempting to pause.

---

## How to Handle Errors

1. **Check the Error Type**: Identify the specific error from the list above.
2. **Understand the Context**: Read the error description to understand what went wrong.
3. **Follow the Fix**: Apply the suggested fix or workaround.
4. **Verify State**: Ensure the system is in the correct state for your operation.
5. **Retry**: After applying the fix, retry your operation.

## Common Error Prevention Tips

-   Always check balances before performing financial operations
-   Verify permissions and ownership before calling restricted functions
-   Ensure subnets are properly registered and active
-   Validate all input parameters before submitting transactions
-   Check the current state of the system before performing state-dependent operations
-   Use proper error handling in your applications to catch and handle these errors gracefully

## Getting Help

If you encounter an error not listed here or need additional help:

1. Check the [IPC Documentation](https://docs.ipc.space/)
2. Review the [IPC GitHub Repository](https://github.com/consensus-shipyard/ipc)
3. Join the [IPC Discord Community](https://discord.gg/ipc)
4. Open an issue on GitHub with detailed error information
