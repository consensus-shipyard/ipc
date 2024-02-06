// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import "./ERC20TokenMessenger.sol"; // Ensure this path is correct
import {SafeERC20} from "openzeppelin-contracts/token/ERC20/utils/SafeERC20.sol";
import {IERC20} from "openzeppelin-contracts/token/ERC20/IERC20.sol";

/**
 * @title TokenTransferAndMint
 * @notice Contract to handle token transfer from L1, lock them and mint on L2.
 */
contract TokenTransferAndMint is ERC20TokenMessenger {
    using SafeERC20 for IERC20;

    address private sourceContract;
    SubnetID private destinationSubnet;
    address private destinationContract;

    function _handleIpcCall(
        IpcEnvelope memory envelope,
        CallMsg memory callMsg
    ) internal override returns (bytes memory) {
        console.log("_handleIpcCall");
        console.logBytes(envelope.message);
        console.log(envelope.value);
        console.log(envelope.nonce);
        //CallMsg memory callMsg = abi.decode(envelope.message, (CallMsg));

        (address receiver, uint256 amount) = abi.decode(callMsg.params, (address, uint256));
        console.log("INFO");
        console.log(receiver);
        console.log(amount);

        return bytes("");
    }

    /**
     * @dev Constructor for TokenTransferAndMint
     * @param _gateway Address of the gateway for cross-network communication
     * @param _sourceContract Address of the source ERC20 token contract
     * @param _destinationSubnet SubnetID of the destination network
     * @param _destinationContract Address of the destination contract for minting
     */
    constructor(
        address _gateway,
        address _sourceContract,
        SubnetID memory _destinationSubnet,
        address _destinationContract
    ) ERC20TokenMessenger(_gateway) {
        sourceContract = _sourceContract;
        destinationSubnet = _destinationSubnet;
        destinationContract = _destinationContract;
    }

    /**
     * @notice Transfers tokens from L1, locks them, and requests minting on L2.
     * @param receiver Address to receive the minted tokens on L2
     * @param amount Amount of tokens to be transferred and minted
     */
    function transferAndMint(address receiver, uint256 amount) external payable {
        // Transfer and lock tokens on L1 using the inherited sendToken function
        _sendToken(sourceContract, destinationSubnet, destinationContract, receiver, amount);
    }

    /* TODO integrate with IpcReceiver */
    function onXNetMessageReceived(address _to, uint256 _amount) public /* parameters */ {
        // Logic to handle IPC xnet message and mint tokens
        address to;
        uint256 amount;
        (to, amount) = extractParameters(/* parameters */ _to, _amount);
        IERC20(sourceContract).safeTransfer(to, amount);
    }

    /* TODO Change code below to parse parameters */
    function extractParameters(/* parameters */ address _to, uint256 _amount) internal view returns (address, uint256) {
        return (_to, _amount);
    }
}
