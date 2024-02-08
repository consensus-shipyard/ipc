// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {SafeERC20} from "openzeppelin-contracts/token/ERC20/utils/SafeERC20.sol";
import {IERC20} from "openzeppelin-contracts/token/ERC20/IERC20.sol";
import {ReentrancyGuard} from "../../lib/LibReentrancyGuard.sol";
import {FvmAddressHelper} from "../../lib/FvmAddressHelper.sol";
import {FvmAddress} from "../../structs/FvmAddress.sol";
import {GatewayMessengerFacet} from "../../gateway/GatewayMessengerFacet.sol";
import {GatewayGetterFacet} from "../../gateway/GatewayGetterFacet.sol";
import {GatewayCannotBeZero, NotEnoughFunds} from "../../errors/IPCErrors.sol";
import {IpcExchange} from "../../../sdk/IpcContract.sol";
import {IpcEnvelope, ResultMsg, CallMsg, IpcMsgKind} from "../../structs/CrossNet.sol";
import {IPCAddress, SubnetID} from "../../structs/Subnet.sol";

error NoTransfer();
error ZeroAddress();

/**
 * @title IpcTokenController
 * @notice Contract to handle token transfer from L1, lock them and mint on L2.
 */
contract IpcTokenController is IpcExchange, ReentrancyGuard {
    using SafeERC20 for IERC20;

    address private tokenContractAddress;
    SubnetID private destinationSubnet;
    address private destinationContract;
    SubnetID public networkName;

    GatewayMessengerFacet private immutable messenger;
    
    uint256 public constant DEFAULT_CROSS_MSG_FEE = 10 gwei;
    uint64 public nonce = 0;

    event TokenSent(
        address tokenContractAddress,
        address sender,
        SubnetID destinationSubnet,
        address destinationContract,
        address receiver,
        uint64 nonce,
        uint256 value
    );

    /**
     * @dev Constructor for IpcTokenController
     * @param _gateway Address of the gateway for cross-network communication
     * @param _tokenContractAddress Address of the source ERC20 token contract
     * @param _destinationSubnet SubnetID of the destination network
     * @param _destinationContract Address of the destination contract for minting
     */
    constructor(
        address _gateway,
        address _tokenContractAddress,
        SubnetID memory _destinationSubnet,
        address _destinationContract
    ) IpcExchange(_gateway) {
        tokenContractAddress = _tokenContractAddress;
        destinationSubnet = _destinationSubnet;
        destinationContract = _destinationContract;

        networkName = GatewayGetterFacet(address(_gateway)).getNetworkName();
        messenger = GatewayMessengerFacet(address(_gateway));
    }


    /**
     * @notice Transfers tokens from L1, locks them, and requests minting on L2.
     * @param receiver Address to receive the minted tokens on L2
     * @param amount Amount of tokens to be transferred and minted
     */
    function depositTokens(address receiver, uint256 amount) external payable {
        // Transfer and lock tokens on L1 using the inherited sendToken function
        _sendToken(tokenContractAddress, destinationSubnet, destinationContract, receiver, amount);
    }

    function depositTokensWithReturn(address receiver, uint256 amount) external payable returns (IpcEnvelope memory) {
        // Transfer and lock tokens on L1 using the inherited sendToken function
        return _sendToken(tokenContractAddress, destinationSubnet, destinationContract, receiver, amount);
    }


    function _handleIpcCall(
        IpcEnvelope memory envelope,
        CallMsg memory callMsg
    ) internal override returns (bytes memory) {
        (address receiver, uint256 amount) = abi.decode(callMsg.params, (address, uint256));
        IERC20(tokenContractAddress).safeTransfer(receiver, amount);
        return bytes("");
    }

    function _sendToken(
        address tokenContractAddress,
        SubnetID memory destinationSubnet,
        address destinationContract,
        address receiver,
        uint256 amount
    ) internal returns (IpcEnvelope memory committed) {
        if (destinationContract == address(0)) {
            revert ZeroAddress();
        }
        if (receiver == address(0)) {
            revert ZeroAddress();
        }
        if (msg.value != DEFAULT_CROSS_MSG_FEE) {
            revert NotEnoughFunds();
        }

        uint64 lastNonce = nonce;

        emit TokenSent({
            tokenContractAddress: tokenContractAddress,
            sender: msg.sender,
            destinationSubnet: destinationSubnet,
            destinationContract: destinationContract,
            receiver: receiver,
            nonce: lastNonce,
            value: amount
        });
        nonce++;

        uint256 startingBalance = IERC20(tokenContractAddress).balanceOf(address(this));
        IERC20(tokenContractAddress).safeTransferFrom({from: msg.sender, to: address(this), value: amount});
        uint256 endingBalance = IERC20(tokenContractAddress).balanceOf(address(this));

        if (endingBalance <= startingBalance) {
            revert NoTransfer();
        }
        CallMsg memory message = CallMsg({
            method: abi.encodePacked(bytes4(keccak256("transfer(address,uint256)"))),
            params: abi.encode(receiver, amount)
        });
        IpcEnvelope memory crossMsg = IpcEnvelope({
            kind: IpcMsgKind.Call,
            from: IPCAddress({subnetId: networkName, rawAddress: FvmAddressHelper.from(tokenContractAddress)}),
            to: IPCAddress({subnetId: destinationSubnet, rawAddress: FvmAddressHelper.from(destinationContract)}),
            value: DEFAULT_CROSS_MSG_FEE,
            nonce: lastNonce,
            message: abi.encode(message)
        });

        return messenger.sendContractXnetMessage{value: DEFAULT_CROSS_MSG_FEE}(crossMsg);
    }

    function _handleIpcResult(
        IpcEnvelope storage original,
        IpcEnvelope memory result,
        ResultMsg memory resultMsg
    ) internal override {}
}
