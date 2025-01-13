## Overview

General cross messages enable the transmission of arbitrary messages between IPC subnets. They are particularly useful for transferring tokens (when supported, more on this later) or transmitting arbitrary data. This is not a feature that allows users to trigger messages from wallets; rather, it is a framework for cross-subnet contract-to-contract communication.

Messages are transmitted across subnets using top-down and bottom-up mechanisms. For example, if a message is sent from a parent to a child subnet, the top-down mechanism is used, and vice versa. Similarly, messages between subnets not directly connected but sharing a common parent are routed automatically by the IPC system.

## Limitations

- Only contracts (not externally owned accounts, or EOAs) can invoke cross messages.
- Token transfers are only allowed if the source subnet, destination subnet, and all intermediate subnets share the same token.
- Since cross messages flow through the entire route between the source and destination subnets, multi-hop messages may take longer to be delivered.

## Usage

Messages are passed via the `Gateway` contract, which exposes the following method for triggering cross messages (only callable by another contract):

```solidity
function sendContractXnetMessage(
    IpcEnvelope calldata envelope
) external payable returns (IpcEnvelope memory committed);
```

The actual message is embedded within the IPC envelope as ABI-encoded bytes.

### IPC Envelope Structure

```solidity
struct IpcEnvelope {
    /// @dev Type of the message being propagated.
    IpcMsgKind kind;
    /// @dev Destination of the message.
    IPCAddress to;
    /// @dev Address of the sender.
    IPCAddress from;
    /// @dev Outgoing nonce for the envelope, ensuring uniqueness per subnet and preventing replay attacks.
    uint64 localNonce;
    /// @dev Value being sent with the message (e.g., tokens).
    uint256 value;
    /// @dev ABI-encoded message payload.
    bytes message;
    /// @dev Original nonce from the source subnet, useful for tracing messages across subnets.
    uint64 originalNonce;
}
```

### Message Types of Interest

- **Call**: Represents a request sent to a destination contract.
- **Result**: Represents the response to a `Call` message, which could either be the result or an error report.

### Contract Requirements

Both sending and receiving contracts must implement the `IpcExchange` abstract contract from the IPC SDK to handle cross messages. Key methods to implement include:

#### Handling Incoming Calls

This must be implemented in the contract receiving the message:

```solidity
function _handleIpcCall(
    IpcEnvelope memory envelope,
    CallMsg memory callMsg
) internal virtual returns (bytes memory);
```

#### Handling Results

This must be implemented in the contract sending the message to handle results from the destination contract:

```solidity
function _handleIpcResult(
    IpcEnvelope storage original,
    IpcEnvelope memory result,
    ResultMsg memory resultMsg
) internal virtual;
```

For an example, refer to [CrossMessengerCaller.sol](../../contracts/contracts/examples/CrossMessengerCaller.sol).

## Results

Results provide either error propagation or responses from the destination contract back to the caller. This creates a request-response mechanism where a `Call` represents the request and a `Result` represents the response.

The IPC envelope will have a `kind` of `Result`, and the message will follow the standardized schema below (see [CrossNet.sol](../../contracts/contracts/structs/CrossNet.sol)):

```solidity
struct ResultMsg {
    /// @dev ID of the envelope the result belongs to.
    bytes32 id;
    /// @dev Outcome of the call (success or type of error).
    OutcomeType outcome;
    /// @dev ABI-encoded return value or failure reason.
    bytes ret;
}
```

### Outcome Types

- **Ok**: Message was delivered successfully, and the `ret` field contains the response from the called contract.
- **SystemErr**: An error occurred during message transmission. The return value indicates the `InvalidXnetMessageReason` (see [IPCErrors.sol](../../contracts/contracts/errors/IPCErrors.sol)).
- **ActorErr**: A custom error returned by the called contract.

## How It Works

1. A custom contract calls `sendContractXnetMessage` on the local `Gateway` with a message inside the IPC envelope (of kind `Call`). The envelope includes the destination subnet ID, which is used for routing.
2. Based on the message route, the gateway determines whether to send it upward (bottom-up) to a parent or downward (top-down) to a child subnet.
3. If the message arrives at an intermediate subnet that is not the destination, it is stored in a postbox for further propagation.
4. Messages in the postbox are processed automatically and routed to their next destination (parent or child subnet).
5. Steps 3 and 4 are repeated until the message either encounters an error or reaches the destination subnet. If an error occurs, a `Result` message is sent back to the source subnet.
6. Upon reaching the destination, the message is executed by invoking a method on the destination contract, and a `Result` message is sent back to the source subnet.
7. Steps 3 and 4 repeat until the `Result` message arrives at the source subnet, where it is executed on the original contract.

## Debugging

During message propagation, the following events are emitted by the subnet gateway, providing useful insights for debugging. Each event includes a unique message ID for cross-subnet tracking:

- **`NewTopDownMessage`**: Emitted when a message is sent downward to a child subnet.
- **`QueuedBottomUpMessage`**: Emitted when a message is prepared for inclusion in a bottom-up checkpoint.
- **`MessageStoredInPostbox`**: Emitted when a message is received by an intermediate subnet and stored for further propagation.
- **`MessagePropagatedFromPostbox`**: Emitted when a message is sent from the postbox to the next subnet.
