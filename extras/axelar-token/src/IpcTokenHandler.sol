// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import { InterchainTokenExecutable } from '@axelar-network/interchain-token-service/executable/InterchainTokenExecutable.sol';
import { IERC20 } from "openzeppelin-contracts/interfaces/IERC20.sol";
import { SubnetID, SupplySource, SupplyKind } from "@ipc/src/structs/Subnet.sol";
import { FvmAddress } from "@ipc/src/structs/FvmAddress.sol";
import { IpcHandler } from "@ipc/sdk/IpcContract.sol";
import { IpcMsgKind, ResultMsg, OutcomeType, IpcEnvelope } from "@ipc/src/structs/CrossNet.sol";
import { FvmAddressHelper } from "@ipc/src/lib/FvmAddressHelper.sol";
import { SubnetIDHelper } from "@ipc/src/lib/SubnetIDHelper.sol";
import { SafeERC20 } from "openzeppelin-contracts/token/ERC20/utils/SafeERC20.sol";

interface TokenFundedGateway {
    function fundWithToken(SubnetID calldata subnetId, FvmAddress calldata to, uint256 amount) external;
}

interface SubnetActor {
    function supplySource() external returns (SupplySource memory supply);
}

// @notice The IpcTokenHandler sits in an Axelar-supported L1 housing an IPC subnet hierarchy. It is invoked by the
//         IpcTokenSender via the Axelar ITS, receiving some token value to deposit into an IPC subnet (specified in the
//         incoming message). The IpcTokenHandler handles deposit failures by crediting the value back to the original
//         beneficiary, and making it available from them to withdraw() on the rootnet.
contract IpcTokenHandler is InterchainTokenExecutable, IpcHandler {
    using FvmAddressHelper for address;
    using FvmAddressHelper for FvmAddress;
    using SubnetIDHelper for SubnetID;
    using SafeERC20 for IERC20;

    error NothingToWithdraw();

    TokenFundedGateway public _ipcGateway;
    mapping(address beneficiary => mapping(address token => uint256 value)) private _claims;

    event SubnetFunded(SubnetID indexed subnet, address indexed recipient, uint256 value);
    event FundingFailed(SubnetID indexed subnet, address indexed recipient, uint256 value);

    constructor(address axelarIts, address ipcGateway) InterchainTokenExecutable(axelarIts) {
        _ipcGateway = TokenFundedGateway(ipcGateway);
    }

    // @notice The InterchainTokenExecutable abstract parent contract hands off to this function after verifying that
    //         the call originated at the Axelar ITS.
    function _executeWithInterchainToken(
        bytes32, // commandId
        string calldata, // sourceChain
        bytes calldata, // sourceAddress
        bytes calldata data,
        bytes32, // tokenId
        address tokenAddr,
        uint256 amount
    ) internal override {
        (SubnetID memory subnet, address recipient) = abi.decode(data, (SubnetID, address));

        IERC20 token = IERC20(tokenAddr);
        require(token.balanceOf(address(this)) >= amount, "insufficient balance");

        // Authorize the IPC gateway to spend these tokens on our behalf.
        token.approve(address(_ipcGateway), amount);

        // Fund the designated subnet via the IPC gateway.
        _ipcGateway.fundWithToken(subnet, recipient.from(), amount);

        // Emit an event.
        emit SubnetFunded(subnet, recipient, amount);
    }

    // @notice Handles result messages for funding operations.
    function handleIpcMessage(IpcEnvelope calldata envelope) external payable returns (bytes memory ret) {
        if (msg.sender != address(_ipcGateway)) {
            revert IpcHandler.CallerIsNotGateway();
        }
        if (envelope.kind != IpcMsgKind.Result) {
            revert IpcHandler.UnsupportedMsgKind();
        }

        ResultMsg memory result = abi.decode(envelope.message, (ResultMsg));
        if (result.outcome != OutcomeType.Ok) {
            // Note: IPC only supports deploying subnets via our blessed registry, so we can trust the code behind
            // the subnet actor.
            SupplySource memory supplySource = SubnetActor(envelope.from.subnetId.getAddress()).supplySource();
            require(supplySource.kind == SupplyKind.ERC20, "expected ERC20 supply source");

            // Results will carry the original beneficiary in the 'from' address.
            address beneficiary = envelope.from.rawAddress.extractEvmAddress();

            // We credit the token funds to the beneficiary. The beneficiary will have to call withdraw() to pull the
            // funds out on this network.
            _claims[beneficiary][supplySource.tokenAddress] += envelope.value;

            // Emit an event.
            emit FundingFailed(envelope.from.subnetId, beneficiary, envelope.value);
        }

        return bytes("");
    }

    // @notice Withdraws all available balance for the specified token for the sender.
    function withdraw(address token) external {
        uint256 available = _claims[msg.sender][token];
        if (available == 0) {
            revert NothingToWithdraw();
        }

        delete _claims[msg.sender][token];
        IERC20(token).safeTransfer(msg.sender, available);
    }

    // @notice Queries the claim of a beneficiary over a particular token.
    function getClaimFor(address beneficiary, address token) external view returns (uint256) {
        return _claims[beneficiary][token];
    }
}