// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {IpcEnvelope, IpcMsgKind, CallMsg} from "../structs/CrossNet.sol";
import {SubnetID, IPCAddress} from "../structs/Subnet.sol";
import {FvmAddressHelper} from "../lib/FvmAddressHelper.sol";
import {CrossMsgHelper} from "../lib/CrossMsgHelper.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {InvalidXnetMessage, InvalidXnetMessageReason} from "../errors/IPCErrors.sol";

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {Pausable} from "@openzeppelin/contracts/utils/Pausable.sol";
import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

/**
 * @title EthGatewayMessenger
 * @notice EVM-native IPC gateway messenger for Ethereum Sepolia.
 *
 * Allows Ethereum smart contracts to send IPC cross-net messages addressed to
 * IPC subnet actors/contracts. The IPC subnet's validator set observes
 * `XnetMessageCommitted` events from this contract and executes the top-down
 * messages.
 *
 * This is a slim EVM-native alternative to the full GatewayDiamond that avoids
 * FVM-specific precompile dependencies (fevmate/FilAddress FVM calls). It provides
 * the `sendContractXnetMessage` interface sufficient for WS-C3 cross-message routing.
 *
 * Security:
 * - Only smart contracts can send xnet messages (EOA check: msg.sender.code.length > 0).
 * - Pausable by owner for emergency stop.
 * - Owner can approve/revoke subnet actor registrations.
 * - Reentrancy-guarded on sendContractXnetMessage.
 */
contract EthGatewayMessenger is Ownable, Pausable, ReentrancyGuard {
    using FvmAddressHelper for address;
    using CrossMsgHelper for IpcEnvelope;
    using SubnetIDHelper for SubnetID;

    // ─── Events ──────────────────────────────────────────────────────────────

    /**
     * @notice Emitted when an IPC cross-net message is committed for dispatch.
     * @dev The IPC subnet validator set subscribes to this event and processes
     *      top-down messages for execution in the subnet.
     */
    event XnetMessageCommitted(IpcEnvelope envelope);

    /// @notice Emitted when a subnet actor is approved or revoked.
    event SubnetApprovalUpdated(address indexed subnet, bool approved);

    // ─── Errors ───────────────────────────────────────────────────────────────

    error CallerIsEOA();
    error SubnetNotApproved(address subnet);

    // ─── State ────────────────────────────────────────────────────────────────

    /// @notice The subnet ID this gateway represents (set at deployment).
    SubnetID public networkName;

    /// @notice Whether subnet actor registration is required for message sending.
    /// When true, only approved subnet contracts can call sendContractXnetMessage.
    /// When false (default for testnet), any contract can send.
    bool public requireApprovedSubnet;

    /// @notice Approved subnet actor contracts.
    mapping(address => bool) public approvedSubnets;

    /// @notice Per-subnet local nonce counter (bumped on each committed message).
    uint64 private _localNonce;

    // ─── Constructor ──────────────────────────────────────────────────────────

    /**
     * @param owner_       Address that receives Ownable + Pausable admin rights.
     * @param networkName_ The IPC SubnetID this gateway belongs to.
     *                     For Ethereum Sepolia: SubnetID{ root: 11155111, route: [] }
     */
    constructor(address owner_, SubnetID memory networkName_) Ownable(owner_) {
        networkName = networkName_;
    }

    // ─── Core: sendContractXnetMessage ────────────────────────────────────────

    /**
     * @notice Send a cross-net message from an Ethereum contract to an IPC subnet actor.
     *
     * Mirrors the IGateway.sendContractXnetMessage interface used by IpcExchange contracts.
     * Emits `XnetMessageCommitted` — the IPC subnet infrastructure processes this event
     * and executes the message as a top-down message in the subnet.
     *
     * @param envelope  The IPC envelope to send. The `from` field will be overwritten with
     *                  the caller's address encoded as an FvmAddress; the nonce fields will
     *                  be set by this contract.
     * @return committed The envelope as committed, with `from`, `localNonce`, and
     *                   `originalNonce` filled in.
     *
     * Requirements:
     * - Caller must be a smart contract (not an EOA).
     * - `envelope.message` must decode as a valid `CallMsg`.
     * - Contract must not be paused.
     * - If `requireApprovedSubnet` is true, caller must be in `approvedSubnets`.
     */
    function sendContractXnetMessage(
        IpcEnvelope memory envelope
    ) external payable whenNotPaused nonReentrant returns (IpcEnvelope memory committed) {
        // Only contracts can send cross-net messages.
        if (msg.sender.code.length == 0) revert CallerIsEOA();

        // Validate envelope message decodes as CallMsg.
        abi.decode(envelope.message, (CallMsg));

        // Optional subnet allowlist check.
        if (requireApprovedSubnet && !approvedSubnets[msg.sender]) {
            revert SubnetNotApproved(msg.sender);
        }

        // Build the committed envelope.
        // - from: caller address encoded as FvmAddress (EAM delegated address, no precompile)
        // - localNonce / originalNonce: assigned by this gateway
        uint64 nonce = _localNonce++;
        committed = IpcEnvelope({
            kind: IpcMsgKind.Call,
            from: IPCAddress({
                subnetId: networkName,
                rawAddress: FvmAddressHelper.from(msg.sender)
            }),
            to: envelope.to,
            value: msg.value,
            message: envelope.message,
            localNonce: nonce,
            originalNonce: nonce
        });

        emit XnetMessageCommitted(committed);
        return committed;
    }

    // ─── Admin: subnet approval ───────────────────────────────────────────────

    /// @notice Approve a subnet contract address to send xnet messages.
    function approveSubnet(address subnet) external onlyOwner {
        approvedSubnets[subnet] = true;
        emit SubnetApprovalUpdated(subnet, true);
    }

    /// @notice Revoke a subnet contract's approval.
    function revokeSubnet(address subnet) external onlyOwner {
        approvedSubnets[subnet] = false;
        emit SubnetApprovalUpdated(subnet, false);
    }

    /// @notice Enable or disable the subnet allowlist check.
    function setRequireApprovedSubnet(bool required) external onlyOwner {
        requireApprovedSubnet = required;
    }

    // ─── Admin: pause / unpause ───────────────────────────────────────────────

    function pause()   external onlyOwner { _pause(); }
    function unpause() external onlyOwner { _unpause(); }

    // ─── Views ────────────────────────────────────────────────────────────────

    /// @notice Current local nonce (next value to be assigned).
    function currentNonce() external view returns (uint64) {
        return _localNonce;
    }
}
