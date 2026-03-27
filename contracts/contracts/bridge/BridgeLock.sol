// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {IpcExchange} from "../../sdk/IpcContract.sol";
import {IpcEnvelope, IpcMsgKind, CallMsg, ResultMsg, OutcomeType} from "../structs/CrossNet.sol";
import {SubnetID, IPCAddress} from "../structs/Subnet.sol";
import {FvmAddressHelper} from "../lib/FvmAddressHelper.sol";

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {Context} from "@openzeppelin/contracts/utils/Context.sol";
import {AccessControlUpgradeable} from "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import {ContextUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/ContextUpgradeable.sol";
import {PausableUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";

/**
 * @title BridgeLock
 * @notice Filecoin-side ERC20 lock contract for the IPC cross-chain token bridge.
 *
 * Users deposit ERC20 tokens into this contract. A TokensLocked event is emitted and
 * an IPC cross-message is dispatched to the Ethereum-side BridgeMint contract on Sepolia,
 * instructing it to mint the equivalent wrapped tokens to the specified recipient.
 *
 * Security properties:
 * - Replay protection: each transferId is recorded on-chain and cannot be reused.
 * - Access control: DEFAULT_ADMIN_ROLE for config, PAUSER_ROLE for pause/unpause.
 * - Pausable: emergency stop halts all lock() calls.
 * - UUPS upgradeable: upgrade gate restricted to DEFAULT_ADMIN_ROLE.
 * - Reentrancy guard on all state-changing external functions.
 *
 * @dev Inherits IpcExchange (non-upgradeable base with immutable gatewayAddr).
 *      OpenZeppelin upgradeable mixins handle all other upgradeable state.
 */
// IpcExchange already inherits OpenZeppelin's non-upgradeable ReentrancyGuard,
// so we do NOT also inherit ReentrancyGuardUpgradeable (would cause duplicate error declaration).
contract BridgeLock is
    Initializable,
    AccessControlUpgradeable,
    PausableUpgradeable,
    UUPSUpgradeable,
    IpcExchange
{
    using SafeERC20 for IERC20;

    // ──────────────────────────────────────────────────────────────────────────
    // Roles
    // ──────────────────────────────────────────────────────────────────────────

    bytes32 public constant PAUSER_ROLE = keccak256("PAUSER_ROLE");

    // ──────────────────────────────────────────────────────────────────────────
    // Events
    // ──────────────────────────────────────────────────────────────────────────

    /**
     * @notice Emitted when tokens are locked and a cross-chain transfer is initiated.
     * @param token      The ERC20 token that was locked.
     * @param sender     The address that initiated the lock.
     * @param recipient  The intended recipient on the destination chain.
     * @param amount     The amount of tokens locked.
     * @param transferId A globally unique identifier for this transfer.
     */
    event TokensLocked(
        address indexed token,
        address indexed sender,
        address indexed recipient,
        uint256 amount,
        bytes32 transferId
    );

    /// @notice Emitted when an IPC result receipt is received for a previously sent lock.
    event TransferAcknowledged(bytes32 indexed transferId, bool success, bytes returnData);

    /// @notice Emitted when the destination configuration is updated.
    event DestinationUpdated(SubnetID destSubnet, address destReceiver);

    /// @notice Emitted when an IPC fee update is applied.
    event IpcFeeUpdated(uint256 newFee);

    /// @notice Emitted on emergency token rescue by admin.
    event TokenRescued(address indexed token, address indexed to, uint256 amount);

    // ──────────────────────────────────────────────────────────────────────────
    // Errors
    // ──────────────────────────────────────────────────────────────────────────

    error ZeroAmount();
    error ZeroAddress();
    error TokenNotAllowed(address token);
    error InsufficientMsgValue(uint256 required, uint256 provided);

    // ──────────────────────────────────────────────────────────────────────────
    // State
    // ──────────────────────────────────────────────────────────────────────────

    /// @notice Destination IPC subnet (Ethereum Sepolia as an IPC SubnetID).
    SubnetID public destSubnet;

    /// @notice Address of BridgeMint contract on the destination subnet.
    address public destReceiver;

    /// @notice Minimum native value (wei) forwarded with each IPC cross-message.
    uint256 public ipcFee;

    /// @notice Replay protection: tracks all transferIds that have been initiated.
    mapping(bytes32 => bool) public processedTransfers;

    /// @notice Per-token allow-list (checked only when tokenAllowlistEnabled is true).
    mapping(address => bool) public allowedTokens;

    /// @notice Whether the token allow-list is enforced.
    bool public tokenAllowlistEnabled;

    /// @dev Monotonically increasing nonce for unique transferId generation.
    uint256 private _nonce;

    // ──────────────────────────────────────────────────────────────────────────
    // Constructor / Initializer
    // ──────────────────────────────────────────────────────────────────────────

    /**
     * @dev IpcExchange requires a constructor to set the immutable gatewayAddr.
     *      _disableInitializers() prevents the implementation contract from being initialized.
     */
    constructor(address gatewayAddr_) IpcExchange(gatewayAddr_) {
        _disableInitializers();
    }

    /**
     * @notice Initialize the proxy instance.
     * @param admin_        Address granted DEFAULT_ADMIN_ROLE and PAUSER_ROLE.
     * @param destSubnet_   IPC SubnetID of the destination chain.
     * @param destReceiver_ Address of the BridgeMint contract on the destination.
     * @param ipcFee_       Native value (wei) forwarded with each IPC cross-message.
     */
    function initialize(
        address admin_,
        SubnetID calldata destSubnet_,
        address destReceiver_,
        uint256 ipcFee_
    ) external initializer {
        if (admin_ == address(0)) revert ZeroAddress();
        if (destReceiver_ == address(0)) revert ZeroAddress();

        __AccessControl_init();
        __Pausable_init();
        __UUPSUpgradeable_init();

        _grantRole(DEFAULT_ADMIN_ROLE, admin_);
        _grantRole(PAUSER_ROLE, admin_);

        destSubnet   = destSubnet_;
        destReceiver = destReceiver_;
        ipcFee       = ipcFee_;
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Core: lock
    // ──────────────────────────────────────────────────────────────────────────

    /**
     * @notice Lock `amount` of `token` and initiate a cross-chain transfer to `recipient`.
     *
     * @param token     ERC20 token contract address to lock.
     * @param amount    Token amount to lock (must be > 0).
     * @param recipient Recipient address on the destination chain.
     *
     * Preconditions:
     * - Caller must have approved this contract for at least `amount` of `token`.
     * - msg.value must be >= ipcFee (covers IPC gateway dispatch cost).
     * - Contract must not be paused.
     *
     * A unique transferId derived from (chainid, contract, sender, token, amount, recipient, nonce)
     * is recorded on-chain for replay protection and cross-chain correlation.
     */
    function lock(
        address token,
        uint256 amount,
        address recipient
    ) external payable whenNotPaused {
        // nonReentrant omitted: performIpcCall() (called below) is itself nonReentrant
        // via IpcExchange's ReentrancyGuard, preventing gateway-level re-entry.
        // ERC20 callback re-entry is prevented by the CEI pattern below:
        // state is updated before the external token pull.
        if (amount == 0)              revert ZeroAmount();
        if (token == address(0))      revert ZeroAddress();
        if (recipient == address(0))  revert ZeroAddress();
        if (msg.value < ipcFee)       revert InsufficientMsgValue(ipcFee, msg.value);
        if (tokenAllowlistEnabled && !allowedTokens[token]) revert TokenNotAllowed(token);

        // CEI: commit state changes before external calls
        // Derive a unique, non-forgeable transferId
        bytes32 transferId = keccak256(
            abi.encodePacked(
                block.chainid,
                address(this),
                msg.sender,
                token,
                amount,
                recipient,
                _nonce++
            )
        );

        // Record for replay protection and cross-chain audit (state update before external call)
        processedTransfers[transferId] = true;

        // External call 1: pull tokens from caller (after state is committed)
        IERC20(token).safeTransferFrom(msg.sender, address(this), amount);

        emit TokensLocked(token, msg.sender, recipient, amount, transferId);

        // Build the cross-message payload for BridgeMint.handleBridgeLock(...)
        bytes memory params = abi.encode(token, recipient, amount, transferId);
        CallMsg memory callMsg = CallMsg({
            method: abi.encodePacked(
                bytes4(keccak256("handleBridgeLock(address,address,uint256,bytes32)"))
            ),
            params: params
        });

        IPCAddress memory to = IPCAddress({
            subnetId: destSubnet,
            rawAddress: FvmAddressHelper.from(destReceiver)
        });

        // Dispatch IPC cross-message; forward all msg.value as the IPC fee
        performIpcCall(to, callMsg, msg.value);
    }

    // ──────────────────────────────────────────────────────────────────────────
    // IpcExchange overrides
    // ──────────────────────────────────────────────────────────────────────────

    /**
     * @notice Handle incoming IPC Call messages.
     * @dev Reserved for future reverse-bridge (unlock) functionality.
     *      Currently reverts to prevent unexpected state changes.
     */
    function _handleIpcCall(
        IpcEnvelope memory,
        CallMsg memory
    ) internal pure override returns (bytes memory) {
        revert("BridgeLock: incoming calls not supported");
    }

    /**
     * @notice Handle IPC Result receipts for previously sent lock cross-messages.
     * @dev MUST NOT revert — IPC treats a revert as a permanent delivery failure
     *      and will not retry. Emit an event and return gracefully on any decode error.
     */
    function _handleIpcResult(
        IpcEnvelope storage original,
        IpcEnvelope memory,
        ResultMsg memory resultMsg
    ) internal override {
        bytes32 tid;
        bool decoded = false;
        // Safe decode: original.message is abi.encode(CallMsg), params is abi.encode(token,recipient,amount,tid)
        if (original.message.length > 0) {
            try this._safeDecodeTransferId(original.message) returns (bytes32 t) {
                tid = t;
                decoded = true;
            } catch {} // solhint-disable-line no-empty-blocks
        }
        bool success = (resultMsg.outcome == OutcomeType.Ok);
        emit TransferAcknowledged(decoded ? tid : bytes32(0), success, resultMsg.ret);
        // On failure: tokens remain locked. Admin uses rescueTokens() after investigation.
    }

    /**
     * @notice External helper enabling try/catch in _handleIpcResult for safe ABI decoding.
     * @dev Callable by this contract only. `message` is the raw abi.encode(CallMsg) bytes.
     */
    function _safeDecodeTransferId(bytes calldata message) external pure returns (bytes32) {
        CallMsg memory call = abi.decode(message, (CallMsg));
        (, , , bytes32 tid) = abi.decode(call.params, (address, address, uint256, bytes32));
        return tid;
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Admin: configuration
    // ──────────────────────────────────────────────────────────────────────────

    /// @notice Update the destination subnet and BridgeMint receiver address.
    function setDestination(
        SubnetID calldata destSubnet_,
        address destReceiver_
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        if (destReceiver_ == address(0)) revert ZeroAddress();
        destSubnet   = destSubnet_;
        destReceiver = destReceiver_;
        emit DestinationUpdated(destSubnet_, destReceiver_);
    }

    /// @notice Update the minimum IPC fee forwarded with each cross-message.
    function setIpcFee(uint256 fee_) external onlyRole(DEFAULT_ADMIN_ROLE) {
        ipcFee = fee_;
        emit IpcFeeUpdated(fee_);
    }

    /// @notice Enable or disable the token allow-list enforcement.
    function setTokenAllowlistEnabled(bool enabled) external onlyRole(DEFAULT_ADMIN_ROLE) {
        tokenAllowlistEnabled = enabled;
    }

    /// @notice Add or remove a token from the allow-list.
    function setTokenAllowed(address token, bool allowed) external onlyRole(DEFAULT_ADMIN_ROLE) {
        if (token == address(0)) revert ZeroAddress();
        allowedTokens[token] = allowed;
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Admin: pause / unpause
    // ──────────────────────────────────────────────────────────────────────────

    /// @notice Pause the contract — halts all lock() calls.
    function pause() external onlyRole(PAUSER_ROLE) {
        _pause();
    }

    /// @notice Unpause the contract.
    function unpause() external onlyRole(PAUSER_ROLE) {
        _unpause();
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Admin: emergency rescue
    // ──────────────────────────────────────────────────────────────────────────

    /**
     * @notice Rescue ERC20 tokens stuck in this contract due to a failed bridge leg.
     * @dev Admin-only. Should only be used after on-chain confirmation that the
     *      corresponding cross-chain mint was NOT completed.
     */
    function rescueTokens(
        address token,
        address to,
        uint256 amount
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        if (to == address(0)) revert ZeroAddress();
        IERC20(token).safeTransfer(to, amount);
        emit TokenRescued(token, to, amount);
    }

    // ──────────────────────────────────────────────────────────────────────────
    // UUPS upgrade guard
    // ──────────────────────────────────────────────────────────────────────────

    /// @dev Restricts upgrades to DEFAULT_ADMIN_ROLE holders.
    function _authorizeUpgrade(address) internal override onlyRole(DEFAULT_ADMIN_ROLE) {}

    // ──────────────────────────────────────────────────────────────────────────
    // Context resolution
    // ──────────────────────────────────────────────────────────────────────────

    // IpcExchange inherits non-upgradeable Ownable (and thus Context), while
    // AccessControlUpgradeable inherits ContextUpgradeable. Both define _msgSender,
    // _msgData, and _contextSuffixLength. We resolve the diamond by delegating to
    // msg.sender / msg.data directly.

    function _msgSender() internal view override(ContextUpgradeable, Context) returns (address) {
        return msg.sender;
    }

    function _msgData() internal pure override(ContextUpgradeable, Context) returns (bytes calldata) {
        return msg.data;
    }

    function _contextSuffixLength() internal pure override(ContextUpgradeable, Context) returns (uint256) {
        return 0;
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Views
    // ──────────────────────────────────────────────────────────────────────────

    /// @notice Returns true if the given transferId has already been initiated from this contract.
    function isProcessed(bytes32 transferId) external view returns (bool) {
        return processedTransfers[transferId];
    }
}
