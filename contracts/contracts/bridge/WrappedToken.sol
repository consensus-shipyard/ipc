// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {ERC20Upgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC20/ERC20Upgradeable.sol";
import {AccessControlUpgradeable} from "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";

/**
 * @title WrappedToken
 * @notice A minimal UUPS-upgradeable ERC20 used as the Ethereum-side representation of a
 *         Filecoin-locked token. Only the designated BridgeMint contract (MINTER_ROLE) can
 *         mint and burn tokens.
 *
 * Deployment: one WrappedToken proxy is deployed per bridged asset.
 */
contract WrappedToken is Initializable, ERC20Upgradeable, AccessControlUpgradeable, UUPSUpgradeable {
    bytes32 public constant MINTER_ROLE = keccak256("MINTER_ROLE");

    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        _disableInitializers();
    }

    /**
     * @param name_   Human-readable token name, e.g. "Wrapped USDC (IPC Bridge)".
     * @param symbol_ Token symbol, e.g. "wUSDC.ipc".
     * @param admin_  Address granted DEFAULT_ADMIN_ROLE and MINTER_ROLE.
     */
    function initialize(string memory name_, string memory symbol_, address admin_) external initializer {
        require(admin_ != address(0), "WrappedToken: zero admin");
        __ERC20_init(name_, symbol_);
        __AccessControl_init();
        __UUPSUpgradeable_init();
        _grantRole(DEFAULT_ADMIN_ROLE, admin_);
        _grantRole(MINTER_ROLE, admin_);
    }

    /// @notice Mint `amount` tokens to `to`. Only MINTER_ROLE.
    function mint(address to, uint256 amount) external onlyRole(MINTER_ROLE) {
        _mint(to, amount);
    }

    /// @notice Burn `amount` tokens from `from`. Only MINTER_ROLE.
    function burn(address from, uint256 amount) external onlyRole(MINTER_ROLE) {
        _burn(from, amount);
    }

    /// @dev Only DEFAULT_ADMIN_ROLE can authorize upgrades.
    function _authorizeUpgrade(address) internal override onlyRole(DEFAULT_ADMIN_ROLE) {}

    // ─── Context resolution ───────────────────────────────────────────────────
    // ERC20Upgradeable and AccessControlUpgradeable both inherit ContextUpgradeable
    // so no diamond here; no override needed.
}
