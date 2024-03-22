// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.23;
import { SafeERC20Upgradeable } from "@openzeppelin/contracts-upgradeable/token/ERC20/utils/SafeERC20Upgradeable.sol";
import {IERC20Upgradeable} from  "@openzeppelin/contracts-upgradeable/token/ERC20/IERC20Upgradeable.sol";
import {LinkedToken} from "./LinkedToken.sol";
import {SubnetID} from "@ipc/src/structs/Subnet.sol";
import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {ERC20Upgradeable} from  "@openzeppelin/contracts-upgradeable/token/ERC20/ERC20Upgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";

/**
 * @title IpcTokenController
 * @notice Contract to handle token transfer from L1, lock them and mint on L2.
 */
contract LinkedTokenReplica is Initializable, LinkedToken, ERC20Upgradeable, UUPSUpgradeable {
    using SafeERC20Upgradeable for IERC20Upgradeable;

    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        _disableInitializers();
    }

    function initialize(
        address gateway,
        address underlyingToken,
        SubnetID memory linkedSubnet,
        address linkedContract
    ) public initializer{
        __LinkedToken_init(gateway, underlyingToken, linkedSubnet, linkedContract);
        __UUPSUpgradeable_init();
        __ERC20_init("USDCTestReplica", "USDCtR");
    }

    function reinitialize(
        address gateway,
        address underlyingToken,
        SubnetID memory linkedSubnet,
        address linkedContract
    ) public reinitializer(2) {
        __LinkedToken_init(gateway, underlyingToken, linkedSubnet, linkedContract);
        __UUPSUpgradeable_init();
        __ERC20_init("USDCTestReplica", "USDCtR");
    }


    // upgrade proxy - onlyOwner can upgrade
    // owner is set in inherited initializer -> __LinkedToken_init -> __IpcExchangeUpgradeable_init
    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}


    function _captureTokens(address holder, uint256 amount) internal override {
        _burn(holder, amount);
    }

    function _releaseTokens(address beneficiary, uint256 amount)
        internal
        override
    {
        _mint(beneficiary, amount);
    }
}
