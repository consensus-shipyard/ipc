// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {IPCAddress, SubnetID} from "../../../src/structs/Subnet.sol";
import {FvmAddress} from "../../../src/structs/FvmAddress.sol";
import {GatewayMessengerFacet} from "../../../src/gateway/GatewayMessengerFacet.sol";
import {GatewayGetterFacet} from "../../../src/gateway/GatewayGetterFacet.sol";
import {CrossMsg, StorableMsg} from "../../../src/structs/Checkpoint.sol";
import {GatewayCannotBeZero, NotEnoughFunds} from "../../../src/errors/IPCErrors.sol";
import {IERC20} from "../../../lib/openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "../../../lib/openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {ReentrancyGuard} from "../../../lib/openzeppelin-contracts/contracts/security/ReentrancyGuard.sol";
import {FvmAddressHelper} from "../../../src/lib/FvmAddressHelper.sol";

error NoTransfer();
error ZeroAddress();

/**
 * @title TokenMessenger
 * @notice An example of a contract that allows users to send a token across subnets.
 */
contract ERC20TokenMessenger is ReentrancyGuard {
    using FvmAddressHelper for FvmAddress;
    using SafeERC20 for IERC20;

    uint64 public nonce = 0;
    // Gateway facet used to send messages
    GatewayMessengerFacet private immutable messenger;
    // Gateway facet used to get information about the subnet
    GatewayGetterFacet private immutable info;
    // Cross-net fee
    uint256 public constant DEFAULT_CROSS_MSG_FEE = 10 gwei;

    event TokenSent(
        address sourceContract,
        address sender,
        SubnetID destinationSubnet,
        address destinationContract,
        address receiver,
        uint64 nonce,
        uint256 value
    );

    constructor(address _gateway) {
        if (_gateway == address(0)) {
            revert GatewayCannotBeZero();
        }
        messenger = GatewayMessengerFacet(address(_gateway));
        info = GatewayGetterFacet(address(_gateway));
    }

    function sendToken(
        address sourceContract,
        SubnetID memory destinationSubnet,
        address destinationContract,
        address receiver,
        uint256 amount
    ) external payable nonReentrant {
        if (destinationContract == address(0)) {
            revert ZeroAddress();
        }
        if (receiver == address(0)) {
            revert ZeroAddress();
        }
        if (msg.value != DEFAULT_CROSS_MSG_FEE ) {
            revert NotEnoughFunds();
        }

        uint64 lastNonce = nonce;

        emit TokenSent({
            sourceContract: sourceContract,
            sender: msg.sender,
            destinationSubnet: destinationSubnet,
            destinationContract: destinationContract,
            receiver: receiver,
            nonce: lastNonce,
            value: amount
        });
        nonce++;

        uint256 startingBalance = IERC20(sourceContract).balanceOf(address(this));

        IERC20(sourceContract).safeTransferFrom({from: msg.sender, to: address(this), value: amount});

        uint256 endingBalance = IERC20(sourceContract).balanceOf(address(this));

        if (endingBalance <= startingBalance) {
            revert NoTransfer();
        }

        CrossMsg memory crossMsg = CrossMsg({
            message: StorableMsg({
                from: IPCAddress({
                    subnetId: info.getNetworkName(),
                    rawAddress: FvmAddressHelper.from(sourceContract)
                }),
                to: IPCAddress({subnetId: destinationSubnet, rawAddress: FvmAddressHelper.from(destinationContract)}),
                value: 0,
                nonce: lastNonce,
                method: bytes4(keccak256("transfer(address,uint256)")),
                params: abi.encode(receiver, amount)
            }),
            wrapped: false
        });

        return messenger.sendCrossMessage{value: msg.value}(crossMsg);
    }

    receive() external payable {}
}