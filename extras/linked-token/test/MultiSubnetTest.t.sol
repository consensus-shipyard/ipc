// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {MultiSubnetTestBase} from "@ipc/test/MultiSubnetTestBase.sol";
import {LinkedTokenController} from "../src/LinkedTokenController.sol";
import {LinkedTokenReplica} from "../src/LinkedTokenReplica.sol";
import {USDCTest} from "../src/USDCTest.sol";

import {
    SubnetID,
    Subnet,
    IPCAddress,
    Validator
} from "@ipc/src/structs/Subnet.sol";
import {FvmAddressHelper} from "@ipc/src/lib/FvmAddressHelper.sol";
import {
    IpcEnvelope,
    BottomUpMsgBatch,
    BottomUpCheckpoint,
    ParentFinality,
    IpcMsgKind,
    ResultMsg,
    CallMsg
} from "@ipc/src/structs/CrossNet.sol";
import {IERC20} from "openzeppelin-contracts/token/ERC20/IERC20.sol";
import {CrossMsgHelper} from "@ipc/src/lib/CrossMsgHelper.sol";
import {IpcHandler} from "@ipc/sdk/IpcContract.sol";

import "forge-std/console.sol";

contract MultiSubnetTest is MultiSubnetTestBase {
    // @dev This test verifies that USDC bridge connects correctly
    // a contract from native subnet with a contract in token subnet through the rootnet.
    using CrossMsgHelper for IpcEnvelope;

    LinkedTokenReplica ipcTokenReplica;
    LinkedTokenController ipcTokenController;

    function testMultiSubnet_Native_FundFromParentToChild_USDCBridge() public {
        IpcEnvelope[] memory msgs = new IpcEnvelope[](1);
        IpcEnvelope memory expected;

        address holder = vm.addr(100);
        address recipient = vm.addr(200);
        address owner = address(this);
        uint256 transferAmount = 300;
        uint256 holderTotalAmount = 1000;

        vm.deal(address(rootTokenSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.prank(address(rootTokenSubnetActor));
        registerSubnetGW(
            DEFAULT_COLLATERAL_AMOUNT,
            address(rootTokenSubnetActor),
            rootGateway
        );

        vm.deal(address(rootNativeSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.prank(address(rootNativeSubnetActor));
        registerSubnetGW(
            DEFAULT_COLLATERAL_AMOUNT,
            address(rootNativeSubnetActor),
            rootGateway
        );

        console.log(
            "--------------- transfer and mint (top-down) ---------------"
        );

        USDCTest testUSDC = new USDCTest();

        testUSDC.mint(100_000);
        testUSDC.transfer(holder, holderTotalAmount);

        require(testUSDC.owner() == owner, "unexpected owner");
        require(
            testUSDC.balanceOf(holder) == holderTotalAmount,
            "unexpected balance"
        );

        // the token replica sits in a native supply child subnet.
        ipcTokenReplica = new LinkedTokenReplica({
            gateway: address(nativeSubnetGateway),
            underlyingToken: address(testUSDC),
            linkedSubnet: rootSubnetName
        });

        // the token controller sits in the root network.
        ipcTokenController = new LinkedTokenController({
            gateway: address(rootGateway),
            underlyingToken: address(testUSDC),
            linkedSubnet: nativeSubnetName
        });
        ipcTokenReplica.initialize(address(ipcTokenController));
        ipcTokenController.initialize(address(ipcTokenReplica));

        vm.prank(holder);
        testUSDC.approve(address(ipcTokenController), transferAmount);

        console.log("mock usdc contract: %s", address(testUSDC));
        console.log("mock usdc owner: %s", owner);
        console.log("mock usdc holder: %s", address(holder));
        console.log("ipcTokenController: %s", address(ipcTokenController));
        console.log(
            "controller allowance for holder: %d",
            testUSDC.allowance(address(holder), address(ipcTokenController))
        );

        vm.prank(address(holder));
        IpcEnvelope memory lockAndTransferEnvelope =
            ipcTokenController.lockAndTransferWithReturn(
                recipient,
                transferAmount
            );

        // Check that the message is in unconfirmedTransfers
        (address receiptSender, uint256 receiptValue) =
            ipcTokenController.getUnconfirmedTransfer(
                lockAndTransferEnvelope.toHash()
            );
        require(
            receiptSender == address(holder),
            "Transfer sender incorrect in unconfirmedTransfers"
        );
        require(
            receiptValue == transferAmount,
            "Transfer amount incorrect in unconfirmedTransfers"
        );

        //confirm that token replica only accept calls to Ipc from the gateway
        vm.prank(owner);
        vm.expectRevert(IpcHandler.CallerIsNotGateway.selector);
        ipcTokenReplica.handleIpcMessage(expected);

        // the message the root gateway's postbox is being executed in the token subnet's gateway

        expected = IpcEnvelope({
            kind: IpcMsgKind.Call,
            from: IPCAddress({
                subnetId: rootSubnetName,
                rawAddress: FvmAddressHelper.from(address(ipcTokenController))
            }),
            to: lockAndTransferEnvelope.to,
            value: 0,
            message: lockAndTransferEnvelope.message,
            nonce: 0 // nonce will be updated by LibGateway.commitCrossMessage
        });

        msgs[0] = expected;
        executeTopDownMsgs(
            msgs,
            nativeSubnetName,
            address(nativeSubnetGateway)
        );

        console.log("fail:");
        console.log(IERC20(ipcTokenReplica).balanceOf(recipient));
        console.log(transferAmount);

        //ensure that tokens are delivered on subnet
        require(
            IERC20(ipcTokenReplica).balanceOf(recipient) == transferAmount,
            "incorrect proxy token balance"
        );

        console.log(
            "--------------- withdraw token (bottom-up)---------------"
        );

        // ensure that USDC holder has initial balance minus tokens previously sent amount of tokens in the root chain
        require(
            holderTotalAmount - transferAmount == testUSDC.balanceOf(holder),
            "unexpected holder balance in withdraw flow"
        );

        vm.prank(recipient);
        expected = ipcTokenReplica.linkedTransfer(holder, transferAmount);

        // check that the message is in unconfirmedTransfers
        (receiptSender, receiptValue) = ipcTokenReplica.getUnconfirmedTransfer(
            expected.toHash()
        );
        require(
            receiptSender == recipient,
            "Transfer sender incorrect in unconfirmedTransfers"
        );
        require(
            receiptValue == transferAmount,
            "Transfer amount incorrect in unconfirmedTransfers"
        );

        console.log("Begin bottom up checkpoint");

        BottomUpCheckpoint memory checkpoint =
            callCreateBottomUpCheckpointFromChildSubnet(
                nativeSubnetName,
                address(nativeSubnetGateway)
            );
        submitBottomUpCheckpoint(checkpoint, address(rootNativeSubnetActor));

        //ensure that usdc tokens are returned on root net
        require(
            holderTotalAmount == testUSDC.balanceOf(holder),
            "unexpected holder balance after withdrawal"
        );
        //ensure that the tokens in the subnet are minted and the token bridge and the usdc holder does not own any
        require(
            0 == ipcTokenReplica.balanceOf(holder),
            "unexpected holder balance in ipcTokenReplica"
        );
        require(
            0 == ipcTokenReplica.balanceOf(address(ipcTokenReplica)),
            "unexpected ipcTokenReplica balance in ipcTokenReplica"
        );
    }
}
