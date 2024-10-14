// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import "forge-std/Test.sol";
import "../../contracts/errors/IPCErrors.sol";
import {EMPTY_BYTES, METHOD_SEND} from "../../contracts/constants/Constants.sol";
import {IpcEnvelope} from "../../contracts/structs/CrossNet.sol";
import {FvmAddress} from "../../contracts/structs/FvmAddress.sol";
import {SubnetID, Subnet, IPCAddress, Validator, Asset, AssetKind} from "../../contracts/structs/Subnet.sol";
import {AssetHelper} from "../../contracts/lib/AssetHelper.sol";
import {SubnetIDHelper} from "../../contracts/lib/SubnetIDHelper.sol";
import {FvmAddressHelper} from "../../contracts/lib/FvmAddressHelper.sol";
import {CrossMsgHelper} from "../../contracts/lib/CrossMsgHelper.sol";
import {GatewayDiamond, FEATURE_MULTILEVEL_CROSSMSG} from "../../contracts/GatewayDiamond.sol";
import {GatewayGetterFacet} from "../../contracts/gateway/GatewayGetterFacet.sol";
import {GatewayManagerFacet} from "../../contracts/gateway/GatewayManagerFacet.sol";
import {XnetMessagingFacet} from "../../contracts/gateway/router/XnetMessagingFacet.sol";
import {DiamondCutFacet} from "../../contracts/diamond/DiamondCutFacet.sol";
import {GatewayMessengerFacet} from "../../contracts/gateway/GatewayMessengerFacet.sol";
import {DiamondLoupeFacet} from "../../contracts/diamond/DiamondLoupeFacet.sol";
import {DiamondCutFacet} from "../../contracts/diamond/DiamondCutFacet.sol";
import {IntegrationTestBase} from "../IntegrationTestBase.sol";
import {L2GatewayActorDiamond} from "../IntegrationTestPresets.sol";
import {TestUtils} from "../helpers/TestUtils.sol";
import {FilAddress} from "fevmate/contracts/utils/FilAddress.sol";

import {GatewayFacetsHelper} from "../helpers/GatewayFacetsHelper.sol";

contract L2GatewayActorDiamondTest is Test, L2GatewayActorDiamond {
    using SubnetIDHelper for SubnetID;
    using CrossMsgHelper for IpcEnvelope;
    using GatewayFacetsHelper for GatewayDiamond;
    using AssetHelper for Asset;

    function testGatewayDiamond_CommitParentFinality_BigNumberOfMessages() public {
        uint256 n = 2000;
        FvmAddress[] memory validators = new FvmAddress[](1);
        validators[0] = FvmAddressHelper.from(vm.addr(100));
        address receipient = vm.addr(102);
        vm.deal(vm.addr(100), 1);

        uint256[] memory weights = new uint[](1);
        weights[0] = 100;

        SubnetID memory id = gatewayDiamond.getter().getNetworkName();

        IpcEnvelope[] memory topDownMsgs = new IpcEnvelope[](n);
        for (uint64 i = 0; i < n; i++) {
            topDownMsgs[i] = TestUtils.newXnetCallMsg(
                IPCAddress({
                    subnetId: gatewayDiamond.getter().getNetworkName().getParentSubnet(),
                    rawAddress: FvmAddressHelper.from(receipient)
                }),
                IPCAddress({subnetId: id, rawAddress: FvmAddressHelper.from(address(this))}),
                0,
                i
            );
        }

        vm.startPrank(FilAddress.SYSTEM_ACTOR);

        gatewayDiamond.xnetMessenger().applyCrossMessages(topDownMsgs);
        require(gatewayDiamond.getter().getSubnetTopDownMsgsLength(id) == 0, "unexpected top-down message");
        (bool ok, uint64 tdn) = gatewayDiamond.getter().getTopDownNonce(id);
        require(!ok && tdn == 0, "unexpected nonce");

        vm.stopPrank();
    }

    function callback() public view {}

    function collateralSource() external pure returns (Asset memory supply) {
        return AssetHelper.native();
    }
}
