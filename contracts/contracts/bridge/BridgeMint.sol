// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {IpcExchange} from "../../sdk/IpcContract.sol";
import {IpcEnvelope, IpcMsgKind, CallMsg, ResultMsg, OutcomeType} from "../structs/CrossNet.sol";
import {SubnetID, IPCAddress} from "../structs/Subnet.sol";
import {FvmAddress} from "../structs/FvmAddress.sol";
import {FvmAddressHelper} from "../lib/FvmAddressHelper.sol";
import {EMPTY_BYTES} from "../constants/Constants.sol";

import {WrappedToken} from "./WrappedToken.sol";

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {Context} from "@openzeppelin/contracts/utils/Context.sol";
import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

import {AccessControlUpgradeable} from "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import {ContextUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/ContextUpgradeable.sol";
import {PausableUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";

/**
 * @title BridgeMint
 * @notice Ethereum-side ERC20 mint/release contract for the IPC cross-chain token bridge.
 *
 * Receives IPC cross-messages from BridgeLock on Filecoin Calibration (via the IPC subnet
 * gateway). On a valid `handleBridgeLock` call, mints wrapped tokens on Ethereum Sepolia
 * to the specified recipient.
 *
 * Security properties:
 * - Caller authentication: only the IPC subnet gateway (gatewayAddr) may call handleIpcMessage.
 *   This is enforced by the `onlyGateway` modifier in IpcExchange.
 * - Origin authentication: the IPC envelope `from` field is checked against the registered
 *   BridgeLock subnet + address. Spoofed messages from unauthorized origins are rejected.
 * - Replay protection: each transferId can only trigger one mint; duplicates revert.
 * - Access control: DEFAULT_ADMIN_ROLE for config; PAUSER_ROLE for pause/unpause.
 * - UUPS upgradeable; upgrade gated to DEFAULT_ADMIN_ROLE.
 * - Pausable: halts all mint operations in an emergency.
 *
 * Asset mapping: each Filecoin token address maps to a deployed WrappedToken proxy on Ethereum.
 * New assets can be registered by admin via `registerAsset()`.
 *
 * @dev Inherits IpcExchange (non-upgradeable, immutable gatewayAddr). The Context diamond
 *      between Ownable (IpcExchange) and AccessControlUpgradeable is resolved explicitly.
 */
contract BridgeMint is
    Initializable,
    AccessControlUpgradeable,
    PausableUpgradeable,
    UUPSUpgradeable,
    IpcExchange
{
    using SafeERC20 for IERC20;
    using FvmAddressHelper for address;

    // ──────────────────────────────────────────────────────────────────────────
    // Roles
    // ──────────────────────────────────────────────────────────────────────────

    bytes32 public constant PAUSER_ROLE = keccak256("PAUSER_ROLE");

    // ──────────────────────────────────────────────────────────────────────────
    // Events
    // ──────────────────────────────────────────────────────────────────────────

    /**
     * @notice Emitted when wrapped tokens are minted to a recipient.
     * @param token       The WrappedToken (Ethereum side) that was minted.
     * @param recipient   The address that received the minted tokens.
     * @param amount      Amount minted.
     * @param transferId  The unique transferId from BridgeLock (for cross-chain correlation).
     */
    event TokensMinted(
        address indexed token,
        address indexed recipient,
        uint256 amount,
        bytes32 indexed transferId
    );

    /// @notice Emitted when a new asset mapping is registered.
    event AssetRegistered(address indexed filecoinToken, address indexed wrappedToken);

    /// @notice Emitted when the authorised BridgeLock origin is updated.
    event BridgeLockOriginUpdated(SubnetID subnetId, address bridgeLock);

    /// @notice Emitted when an IPC message is rejected (wrong origin or replay).
    event MessageRejected(bytes32 indexed transferId, string reason);

    /// @notice Emitted on token rescue by admin.
    event TokenRescued(address indexed token, address indexed to, uint256 amount);

    // ──────────────────────────────────────────────────────────────────────────
    // Errors
    // ──────────────────────────────────────────────────────────────────────────

    error UnauthorizedOrigin();
    error DuplicateTransfer(bytes32 transferId);
    error AssetNotRegistered(address filecoinToken);
    error ZeroAddress();
    error ZeroAmount();

    // ──────────────────────────────────────────────────────────────────────────
    // State
    // ──────────────────────────────────────────────────────────────────────────

    /// @notice The authorised source subnet for BridgeLock messages.
    SubnetID public bridgeLockSubnet;

    /// @notice The authorised BridgeLock contract address on the source subnet.
    address public bridgeLockAddr;

    /// @notice filecoinToken → wrappedToken mapping.
    mapping(address => address) public wrappedTokens;

    /// @notice Replay protection: transferIds that have already been processed.
    mapping(bytes32 => bool) public processedTransfers;

    // ──────────────────────────────────────────────────────────────────────────
    // Constructor / Initializer
    // ──────────────────────────────────────────────────────────────────────────

    /**
     * @dev IpcExchange requires a constructor for the immutable gatewayAddr.
     *      _disableInitializers() prevents the implementation from being initialized directly.
     */
    constructor(address gatewayAddr_) IpcExchange(gatewayAddr_) {
        _disableInitializers();
    }

    /**
     * @notice Initialize the proxy instance.
     * @param admin_            Address granted DEFAULT_ADMIN_ROLE and PAUSER_ROLE.
     * @param bridgeLockSubnet_ IPC SubnetID of the source chain (Filecoin Calibration).
     * @param bridgeLockAddr_   Address of BridgeLock on the source subnet.
     */
    function initialize(
        address admin_,
        SubnetID calldata bridgeLockSubnet_,
        address bridgeLockAddr_
    ) external initializer {
        if (admin_ == address(0)) revert ZeroAddress();
        if (bridgeLockAddr_ == address(0)) revert ZeroAddress();

        __AccessControl_init();
        __Pausable_init();
        __UUPSUpgradeable_init();

        _grantRole(DEFAULT_ADMIN_ROLE, admin_);
        _grantRole(PAUSER_ROLE, admin_);

        bridgeLockSubnet = bridgeLockSubnet_;
        bridgeLockAddr   = bridgeLockAddr_;
    }

    // ──────────────────────────────────────────────────────────────────────────
    // IpcExchange: receive and dispatch IPC calls
    // ──────────────────────────────────────────────────────────────────────────

    /**
     * @notice Entry-point for IPC Call messages delivered by the gateway.
     *
     * Only the gateway may call this (enforced by IpcExchange.onlyGateway modifier).
     * The envelope's `from` field is validated against the registered BridgeLock origin.
     * The method selector in the CallMsg is matched against `handleBridgeLock`.
     *
     * On success, mints wrapped tokens to the recipient and returns EMPTY_BYTES.
     * On origin mismatch or duplicate transferId, reverts so IPC can propagate the error.
     */
    function _handleIpcCall(
        IpcEnvelope memory envelope,
        CallMsg memory callMsg
    ) internal override whenNotPaused returns (bytes memory) {
        // ── 1. Validate origin ────────────────────────────────────────────────
        _validateOrigin(envelope.from);

        // ── 2. Dispatch on method selector ───────────────────────────────────
        bytes4 selector = bytes4(callMsg.method);
        bytes4 expectedSelector = bytes4(keccak256("handleBridgeLock(address,address,uint256,bytes32)"));

        if (selector == expectedSelector) {
            return _handleBridgeLock(callMsg.params);
        }

        revert("BridgeMint: unknown method");
    }

    /**
     * @notice Handle result receipts (not expected in this direction; log and ignore).
     * @dev Must not revert.
     */
    function _handleIpcResult(
        IpcEnvelope storage,
        IpcEnvelope memory,
        ResultMsg memory
    ) internal pure override {
        // BridgeMint does not initiate outbound IPC calls in the current implementation.
        // Results are not expected; silently ignore to avoid blocking IPC.
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Core: handleBridgeLock (minting logic)
    // ──────────────────────────────────────────────────────────────────────────

    /**
     * @notice Decode a BridgeLock payload and mint wrapped tokens to the recipient.
     * @dev Called internally from _handleIpcCall after origin and selector validation.
     *
     * Payload format: abi.encode(filecoinToken, recipient, amount, transferId)
     *   - filecoinToken: ERC20 address on Filecoin (used to look up the wrapped token)
     *   - recipient:     Ethereum address to receive minted tokens
     *   - amount:        Number of tokens to mint (must match amount locked on Filecoin)
     *   - transferId:    Unique id from BridgeLock; rejected if already seen (replay protection)
     */
    function _handleBridgeLock(bytes memory params) internal returns (bytes memory) {
        (address filecoinToken, address recipient, uint256 amount, bytes32 transferId) =
            abi.decode(params, (address, address, uint256, bytes32));

        // Replay protection
        if (processedTransfers[transferId]) revert DuplicateTransfer(transferId);

        // Asset mapping
        address wrapped = wrappedTokens[filecoinToken];
        if (wrapped == address(0)) revert AssetNotRegistered(filecoinToken);

        // Validation
        if (recipient == address(0)) revert ZeroAddress();
        if (amount == 0) revert ZeroAmount();

        // Record before external call (CEI)
        processedTransfers[transferId] = true;

        // Mint
        WrappedToken(wrapped).mint(recipient, amount);

        emit TokensMinted(wrapped, recipient, amount, transferId);

        return EMPTY_BYTES;
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Origin validation
    // ──────────────────────────────────────────────────────────────────────────

    /**
     * @notice Validate that the IPC message originates from the registered BridgeLock.
     * @dev Compares the `from` IPCAddress against (bridgeLockSubnet, bridgeLockAddr).
     *      Uses FvmAddressHelper to encode and compare the Ethereum address in FvmAddress form.
     */
    function _validateOrigin(IPCAddress memory from) internal view {
        // Compare subnet IDs
        if (!_subnetIdEq(from.subnetId, bridgeLockSubnet)) revert UnauthorizedOrigin();

        // Compare address — encode expected address the same way BridgeLock does
        FvmAddress memory expected = FvmAddressHelper.from(bridgeLockAddr);
        if (!FvmAddressHelper.equal(from.rawAddress, expected)) revert UnauthorizedOrigin();
    }

    /**
     * @notice Compare two SubnetIDs for equality.
     * @dev Checks root chainid and route array element-by-element.
     */
    function _subnetIdEq(SubnetID memory a, SubnetID memory b) internal pure returns (bool) {
        if (a.root != b.root) return false;
        if (a.route.length != b.route.length) return false;
        for (uint256 i = 0; i < a.route.length; i++) {
            if (a.route[i] != b.route[i]) return false;
        }
        return true;
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Admin: asset management
    // ──────────────────────────────────────────────────────────────────────────

    /**
     * @notice Register a mapping from a Filecoin token address to an existing WrappedToken.
     * @dev Admin must have already deployed (or have access to) the WrappedToken proxy,
     *      and must grant MINTER_ROLE to this contract on the WrappedToken.
     */
    function registerAsset(
        address filecoinToken,
        address wrappedToken
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        if (filecoinToken == address(0)) revert ZeroAddress();
        if (wrappedToken == address(0)) revert ZeroAddress();
        wrappedTokens[filecoinToken] = wrappedToken;
        emit AssetRegistered(filecoinToken, wrappedToken);
    }

    /**
     * @notice Deploy a new WrappedToken proxy and register it for a Filecoin token.
     * @dev Convenience function: deploys a WrappedToken ERC1967 proxy, grants MINTER_ROLE
     *      to this contract, and registers the mapping.
     * @param filecoinToken  The Filecoin-side ERC20 address (used as the mapping key).
     * @param name           Name for the new WrappedToken, e.g. "Wrapped USDC (IPC Bridge)".
     * @param symbol         Symbol for the new WrappedToken, e.g. "wUSDC.ipc".
     * @param implAddr       Address of the deployed WrappedToken implementation contract.
     * @return wrappedToken  Address of the newly deployed proxy.
     */
    function deployAndRegisterAsset(
        address filecoinToken,
        string calldata name,
        string calldata symbol,
        address implAddr
    ) external onlyRole(DEFAULT_ADMIN_ROLE) returns (address wrappedToken) {
        if (filecoinToken == address(0)) revert ZeroAddress();
        if (implAddr == address(0)) revert ZeroAddress();

        // Deploy proxy with admin = address(this) so we can grant MINTER_ROLE
        bytes memory initData = abi.encodeWithSelector(
            WrappedToken.initialize.selector,
            name,
            symbol,
            address(this)
        );
        wrappedToken = address(new ERC1967Proxy(implAddr, initData));

        wrappedTokens[filecoinToken] = wrappedToken;
        emit AssetRegistered(filecoinToken, wrappedToken);
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Admin: origin management
    // ──────────────────────────────────────────────────────────────────────────

    /// @notice Update the authorised BridgeLock origin.
    function setBridgeLockOrigin(
        SubnetID calldata subnetId,
        address bridgeLockAddr_
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        if (bridgeLockAddr_ == address(0)) revert ZeroAddress();
        bridgeLockSubnet = subnetId;
        bridgeLockAddr   = bridgeLockAddr_;
        emit BridgeLockOriginUpdated(subnetId, bridgeLockAddr_);
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Admin: pause / unpause
    // ──────────────────────────────────────────────────────────────────────────

    /// @notice Pause minting.
    function pause() external onlyRole(PAUSER_ROLE) { _pause(); }

    /// @notice Unpause minting.
    function unpause() external onlyRole(PAUSER_ROLE) { _unpause(); }

    // ──────────────────────────────────────────────────────────────────────────
    // Admin: emergency rescue
    // ──────────────────────────────────────────────────────────────────────────

    /**
     * @notice Rescue ERC20 tokens accidentally sent to this contract.
     * @dev BridgeMint does not hold user tokens under normal operation
     *      (it mints and burns via WrappedToken). Rescue is for edge cases only.
     */
    function rescueTokens(address token, address to, uint256 amount)
        external onlyRole(DEFAULT_ADMIN_ROLE)
    {
        if (to == address(0)) revert ZeroAddress();
        IERC20(token).safeTransfer(to, amount);
        emit TokenRescued(token, to, amount);
    }

    // ──────────────────────────────────────────────────────────────────────────
    // UUPS upgrade guard
    // ──────────────────────────────────────────────────────────────────────────

    function _authorizeUpgrade(address) internal override onlyRole(DEFAULT_ADMIN_ROLE) {}

    // ──────────────────────────────────────────────────────────────────────────
    // Context resolution (Ownable vs AccessControlUpgradeable diamond)
    // ──────────────────────────────────────────────────────────────────────────

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

    /// @notice Returns the wrapped token address for a given Filecoin token, or address(0) if not registered.
    function getWrappedToken(address filecoinToken) external view returns (address) {
        return wrappedTokens[filecoinToken];
    }

    /// @notice Returns true if the transferId has already been processed.
    function isProcessed(bytes32 transferId) external view returns (bool) {
        return processedTransfers[transferId];
    }
}
