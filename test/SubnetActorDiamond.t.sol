// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {Test} from "forge-std/Test.sol";
import {console} from "forge-std/console.sol";
import "../src/errors/IPCErrors.sol";
import {TestUtils} from "./TestUtils.sol";
import {EMPTY_BYTES, METHOD_SEND, EMPTY_HASH} from "../src/constants/Constants.sol";
import {ConsensusType} from "../src/enums/ConsensusType.sol";
import {Status} from "../src/enums/Status.sol";
import {CrossMsg, BottomUpCheckpoint, StorableMsg} from "../src/structs/Checkpoint.sol";
import {FvmAddress} from "../src/structs/FvmAddress.sol";
import {SubnetID, IPCAddress, Subnet, ValidatorInfo} from "../src/structs/Subnet.sol";
import {StorableMsg} from "../src/structs/Checkpoint.sol";
import {IGateway} from "../src/interfaces/IGateway.sol";
import {IDiamond} from "../src/interfaces/IDiamond.sol";
import {IDiamondCut} from "../src/interfaces/IDiamondCut.sol";
import {FvmAddressHelper} from "../src/lib/FvmAddressHelper.sol";
import {CheckpointHelper} from "../src/lib/CheckpointHelper.sol";
import {StorableMsgHelper} from "../src/lib/StorableMsgHelper.sol";
import {SubnetIDHelper} from "../src/lib/SubnetIDHelper.sol";
import {SubnetActorDiamond} from "../src/SubnetActorDiamond.sol";
import {SubnetActorManagerFacet} from "../src/subnet/SubnetActorManagerFacet.sol";
import {SubnetActorGetterFacet} from "../src/subnet/SubnetActorGetterFacet.sol";
import {FilAddress} from "fevmate/utils/FilAddress.sol";
import {LibStaking} from "../src/lib/LibStaking.sol";

import {DefaultGatewayMock} from "./subnetActorMock/DefaultGatewayMock.sol";
import {SubnetManagerTestUtil} from "./subnetActorMock/SubnetManagerTestUtil.sol";

contract SubnetActorDiamondTest is Test {
    using SubnetIDHelper for SubnetID;
    using CheckpointHelper for BottomUpCheckpoint;
    using FilAddress for address;
    using FvmAddressHelper for FvmAddress;

    address private constant DEFAULT_IPC_GATEWAY_ADDR = address(1024);
    uint64 private constant DEFAULT_CHECKPOINT_PERIOD = 10;
    uint256 private constant DEFAULT_MIN_VALIDATOR_STAKE = 1 ether;
    uint64 private constant DEFAULT_MIN_VALIDATORS = 1;
    string private constant DEFAULT_NET_ADDR = "netAddr";
    uint256 private constant CROSS_MSG_FEE = 10 gwei;
    uint256 private constant DEFAULT_RELAYER_REWARD = 10 gwei;
    uint8 private constant DEFAULT_MAJORITY_PERCENTAGE = 70;
    uint64 private constant ROOTNET_CHAINID = 123;

    address private gatewayAddress;
    IGateway private gatewayContract;

    bytes4[] private saGetterSelectors;
    bytes4[] private saManagerSelectors;

    SubnetActorDiamond private saDiamond;
    SubnetManagerTestUtil private saManager;
    SubnetActorGetterFacet private saGetter;

    constructor() {
        saGetterSelectors = TestUtils.generateSelectors(vm, "SubnetActorGetterFacet");
        saManagerSelectors = TestUtils.generateSelectors(vm, "SubnetManagerTestUtil");
    }

    function createSubnetActorDiamondWithFaucets(
        SubnetActorDiamond.ConstructorParams memory params,
        address getterFaucet,
        address managerFaucet
    ) public returns (SubnetActorDiamond) {
        IDiamond.FacetCut[] memory diamondCut = new IDiamond.FacetCut[](2);

        diamondCut[0] = (
            IDiamond.FacetCut({
                facetAddress: getterFaucet,
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: saGetterSelectors
            })
        );

        diamondCut[1] = (
            IDiamond.FacetCut({
                facetAddress: managerFaucet,
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: saManagerSelectors
            })
        );

        saDiamond = new SubnetActorDiamond(diamondCut, params);
        return saDiamond;
    }

    function setUp() public {
        gatewayContract = new DefaultGatewayMock();
        gatewayAddress = address(gatewayContract);

        _assertDeploySubnetActor(
            gatewayAddress,
            ConsensusType.Fendermint,
            DEFAULT_MIN_VALIDATOR_STAKE,
            DEFAULT_MIN_VALIDATORS,
            DEFAULT_CHECKPOINT_PERIOD,
            DEFAULT_MAJORITY_PERCENTAGE
        );
    }

    function ensureBytesEqual(bytes memory _a, bytes memory _b) internal pure {
        require(_a.length == _b.length, "bytes len not equal");
        require(keccak256(_a) == keccak256(_b), "bytes not equal");
    }

    function deriveValidatorAddress(uint8 seq) internal pure returns (address addr, bytes memory data) {
        data = new bytes(65);
        data[1] = bytes1(seq);

        // use data[1:] for the hash
        bytes memory dataSubset = new bytes(data.length - 1);
        for (uint i = 1; i < data.length; i++) {
            dataSubset[i - 1] = data[i];
        }

        addr = address(uint160(uint256(keccak256(dataSubset))));
    }

    function testSubnetActorDiamond_Deployment_Works(
        address _ipcGatewayAddr,
        uint256 _minActivationCollateral,
        uint64 _minValidators,
        uint64 _checkPeriod,
        uint8 _majorityPercentage
    ) public {
        vm.assume(_minActivationCollateral > DEFAULT_MIN_VALIDATOR_STAKE);
        vm.assume(_checkPeriod > DEFAULT_CHECKPOINT_PERIOD);
        vm.assume(_majorityPercentage > 51);
        vm.assume(_majorityPercentage <= 100);
        vm.assume(_ipcGatewayAddr != address(0));

        _assertDeploySubnetActor(
            _ipcGatewayAddr,
            ConsensusType.Fendermint,
            _minActivationCollateral,
            _minValidators,
            _checkPeriod,
            _majorityPercentage
        );

        SubnetID memory parent = saGetter.getParent();
        require(parent.isRoot(), "parent.isRoot()");

        require(saGetter.bottomUpCheckPeriod() == _checkPeriod, "bottomUpCheckPeriod");
    }

    function testSubnetActorDiamond_Deployments_Fail_GatewayCannotBeZero() public {
        SubnetManagerTestUtil saDupMangerFaucet = new SubnetManagerTestUtil();
        SubnetActorGetterFacet saDupGetterFaucet = new SubnetActorGetterFacet();

        vm.expectRevert(GatewayCannotBeZero.selector);
        createSubnetActorDiamondWithFaucets(
            SubnetActorDiamond.ConstructorParams({
                parentId: SubnetID(ROOTNET_CHAINID, new address[](0)),
                ipcGatewayAddr: address(0),
                consensus: ConsensusType.Fendermint,
                minActivationCollateral: DEFAULT_MIN_VALIDATOR_STAKE,
                minValidators: DEFAULT_MIN_VALIDATORS,
                bottomUpCheckPeriod: DEFAULT_CHECKPOINT_PERIOD,
                majorityPercentage: DEFAULT_MAJORITY_PERCENTAGE,
                activeValidatorsLimit: 100,
                powerScale: 12,
                minCrossMsgFee: CROSS_MSG_FEE
            }),
            address(saDupGetterFaucet),
            address(saDupMangerFaucet)
        );
    }

    function testSubnetActorDiamond_Join_Fail_NotOwnerOfPublicKey() public {
        address validator = vm.addr(100);

        vm.deal(validator, 1 gwei);
        vm.prank(validator);
        vm.expectRevert(NotOwnerOfPublicKey.selector);

        saManager.join{value: 10}(new bytes(65));
    }

    function testSubnetActorDiamond_Join_Fail_ZeroColalteral() public {
        (address validator, bytes memory publicKey) = deriveValidatorAddress(100);

        vm.deal(validator, 1 gwei);
        vm.prank(validator);
        vm.expectRevert(CollateralIsZero.selector);

        saManager.join(publicKey);
    }

    function testSubnetActorDiamond_Bootstrap_Node() public {
        (address validator, bytes memory publicKey) = deriveValidatorAddress(100);

        vm.deal(validator, 10 gwei);
        vm.prank(validator);
        saManager.join{value: 10}(publicKey);

        // validator adds a node
        vm.prank(validator);
        saManager.addBootstrapNode("1.2.3.4");

        // not-validator adds a node
        vm.prank(vm.addr(200));
        vm.expectRevert(NotValidator.selector);
        saManager.addBootstrapNode("3.4.5.6");

        string[] memory nodes = saGetter.getBootstrapNodes();
        require(nodes.length == 1, "it returns one node");
        require(
            keccak256(abi.encodePacked((nodes[0]))) == keccak256(abi.encodePacked(("1.2.3.4"))),
            "it returns correct address"
        );

        vm.prank(validator);
        saManager.leave();

        nodes = saGetter.getBootstrapNodes();
        require(nodes.length == 0, "no nodes");
    }

    /// @notice Testing the basic join, stake, leave lifecycle of validators
    function testSubnetActorDiamond_BasicLifeCycle() public {
        (address validator1, bytes memory publicKey1) = deriveValidatorAddress(100);
        (address validator2, bytes memory publicKey2) = deriveValidatorAddress(101);

        uint256 collateral = DEFAULT_MIN_VALIDATOR_STAKE;

        // ======== Step. Join ======
        // initial validator joins
        vm.startPrank(validator1);
        vm.deal(validator1, collateral);

        saManager.join{value: collateral}(publicKey1);

        // collateral confirmed immediately and network boostrapped
        ValidatorInfo memory v = saGetter.getValidator(validator1);
        require(v.totalCollateral == collateral, "total collateral not expected");
        require(v.confirmedCollateral == collateral, "confirmed collateral not 0");
        ensureBytesEqual(v.metadata, publicKey1);
        require(saGetter.bootstrapped(), "subnet not bootstrapped");
        require(saGetter.genesisValidators().length == 1, "genesis validators not 1");

        (uint64 nextConfigNum, uint64 startConfigNum) = saGetter.getConfigurationNumbers();
        require(nextConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER, "next config num not 1");
        require(startConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER, "start config num not 1");

        // second validator joins
        vm.startPrank(validator2);
        vm.deal(validator2, collateral);

        // subnet bootstrapped and should go through queue
        saManager.join{value: collateral}(publicKey2);

        // collateral not confirmed yet
        v = saGetter.getValidator(validator2);
        require(v.totalCollateral == collateral, "total collateral not expected");
        require(v.confirmedCollateral == 0, "confirmed collateral not 0");
        ensureBytesEqual(v.metadata, new bytes(0));

        (nextConfigNum, startConfigNum) = saGetter.getConfigurationNumbers();
        // join will update the metadata, incr by 1
        // join will call deposit, incre by 1
        require(nextConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER + 2, "next config num not 3");
        require(startConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER, "start config num not 1");

        // ======== Step. Confirm join operation ======
        saManager.confirmChange(LibStaking.INITIAL_CONFIGURATION_NUMBER + 1);

        v = saGetter.getValidator(validator2);
        require(v.totalCollateral == collateral, "total collateral not expected after confirm join");
        require(
            v.confirmedCollateral == DEFAULT_MIN_VALIDATOR_STAKE,
            "confirmed collateral not expected after confrim join"
        );

        (nextConfigNum, startConfigNum) = saGetter.getConfigurationNumbers();
        require(
            nextConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER + 2,
            "next config num not 3 after confirm join"
        );
        require(
            startConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER + 2,
            "start config num not 3 after confirm join"
        );

        // ======== Step. Stake more ======
        vm.startPrank(validator1);
        vm.deal(validator1, 10);

        saManager.stake{value: 10}();

        collateral += 10;
        v = saGetter.getValidator(validator1);
        require(v.totalCollateral == collateral, "total collateral not expected after stake");
        require(v.confirmedCollateral == DEFAULT_MIN_VALIDATOR_STAKE, "confirmed collateral not 0 after stake");

        (nextConfigNum, startConfigNum) = saGetter.getConfigurationNumbers();
        require(nextConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER + 3, "next config num not 4 after stake");
        require(startConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER + 2, "start config num not 3 after stake");

        // ======== Step. Confirm stake operation ======
        saManager.confirmChange(LibStaking.INITIAL_CONFIGURATION_NUMBER + 2);

        v = saGetter.getValidator(validator1);
        require(v.totalCollateral == collateral, "total collateral not expected after confirm stake");
        require(v.confirmedCollateral == collateral, "confirmed collateral not expected after confrim stake");

        (nextConfigNum, startConfigNum) = saGetter.getConfigurationNumbers();
        require(
            nextConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER + 3,
            "next config num not 4 after confirm stake"
        );
        require(
            startConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER + 3,
            "start config num not 4 after confirm stake"
        );
        require(saGetter.genesisValidators().length == 1, "genesis validators still 1");

        // ======== Step. Leave ======
        vm.startPrank(validator1);
        saManager.leave();

        v = saGetter.getValidator(validator1);
        require(v.totalCollateral == 0, "total collateral not 0 after confirm leave");
        require(v.confirmedCollateral == collateral, "confirmed collateral not 0 after confrim leave");

        (nextConfigNum, startConfigNum) = saGetter.getConfigurationNumbers();
        require(
            nextConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER + 4,
            "next config num not 5 after confirm leave"
        );
        require(
            startConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER + 3,
            "start config num not 4 after confirm leave"
        );

        // ======== Step. Confirm leave ======
        saManager.confirmChange(LibStaking.INITIAL_CONFIGURATION_NUMBER + 3);

        v = saGetter.getValidator(validator1);
        require(v.totalCollateral == 0, "total collateral not 0 after confirm leave");
        require(v.confirmedCollateral == 0, "confirmed collateral not 0 after confrim leave");

        (nextConfigNum, startConfigNum) = saGetter.getConfigurationNumbers();
        require(
            nextConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER + 4,
            "next config num not 5 after confirm leave"
        );
        require(
            startConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER + 4,
            "start config num not 5 after confirm leave"
        );
    }

    function testSubnetActorDiamond_Unstake() public {
        (address validator, bytes memory publicKey) = deriveValidatorAddress(100);

        vm.prank(validator);
        vm.expectRevert(NotValidator.selector);
        saManager.unstake(10);

        vm.deal(validator, 10 gwei);
        vm.prank(validator);
        saManager.join{value: 10}(publicKey);
        require(saGetter.getValidator(validator).totalCollateral == 10, "initial collateral correct");

        vm.prank(validator);
        vm.expectRevert(NotEnoughCollateral.selector);
        saManager.unstake(100);

        vm.prank(validator);
        vm.expectRevert(NotEnoughCollateral.selector);
        saManager.unstake(10);

        vm.prank(validator);
        saManager.unstake(5);
        require(saGetter.getValidator(validator).totalCollateral == 5, "collateral correct after unstaking");
    }

    // function testSubnetActorDiamond_MultipleJoins_Works_GetValidators() public {
    //     address validator1 = vm.addr(1231);
    //     address validator2 = vm.addr(1232);
    //     address validator3 = vm.addr(1233);
    //     address validator4 = vm.addr(1234);
    //     address validator5 = vm.addr(1235);
    //     address validator6 = vm.addr(1236);
    //     address validator7 = vm.addr(1237);

    //     _assertJoin(validator1, DEFAULT_MIN_VALIDATOR_STAKE);
    //     _assertJoin(validator2, DEFAULT_MIN_VALIDATOR_STAKE);
    //     _assertJoin(validator3, DEFAULT_MIN_VALIDATOR_STAKE);
    //     _assertJoin(validator4, DEFAULT_MIN_VALIDATOR_STAKE);
    //     _assertJoin(validator5, DEFAULT_MIN_VALIDATOR_STAKE);
    //     _assertJoin(validator6, DEFAULT_MIN_VALIDATOR_STAKE);
    //     _assertJoin(validator7, DEFAULT_MIN_VALIDATOR_STAKE);

    //     require(saGetter.validatorCount() == 7);
    //     require(saGetter.getValidators().length == 7);
    //     require(saGetter.getValidatorSet().validators.length == 7);

    //     address[] memory result;
    //     uint256 offset;

    //     (result, offset) = saGetter.getRangeOfValidators(0, 2);
    //     require(result.length == 2);
    //     require(offset == 2);

    //     (result, offset) = saGetter.getRangeOfValidators(0, 0);
    //     require(result.length == 0);
    //     require(offset == 0);

    //     (result, offset) = saGetter.getRangeOfValidators(10, 0);
    //     require(result.length == 0);
    //     require(offset == 0);

    //     (result, offset) = saGetter.getRangeOfValidators(2, 4);
    //     require(result.length == 4);
    //     require(offset == 6);

    //     (result, offset) = saGetter.getRangeOfValidators(2, 0);
    //     require(result.length == 0);
    //     require(offset == 0);

    //     (result, offset) = saGetter.getRangeOfValidators(6, 10);
    //     require(result.length == 1);
    //     require(offset == 7);

    //     (result, offset) = saGetter.getRangeOfValidators(10, 10);
    //     require(result.length == 0);
    //     require(offset == 0);
    // }

    // function testSubnetActorDiamond_MultipleJoins_Fuzz_GetValidators(uint256 offset, uint256 limit, uint256 n) public {
    //     offset = bound(offset, 0, 10);
    //     limit = bound(limit, 0, 10);
    //     n = bound(n, 0, 10);

    //     console.log("fuzz data:");
    //     console.log(offset);
    //     console.log(limit);
    //     console.log(n);

    //     for (uint256 i = 0; i < n; i++) {
    //         address validator = vm.addr(i + 1000);
    //         _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE);
    //     }

    //     require(saGetter.validatorCount() == n);
    //     require(saGetter.getValidators().length == n);
    //     require(saGetter.getValidatorSet().validators.length == n);

    //     address[] memory result;
    //     uint256 newOffset;

    //     (result, newOffset) = saGetter.getRangeOfValidators(offset, limit);
    //     if (limit == 0 || n <= offset) {
    //         require(result.length == 0, "result.length == 0");
    //     } else {
    //         if (limit > n - offset) {
    //             limit = n - offset;
    //         }
    //         require(result.length == limit, "result.length == limit");
    //     }
    // }

    // function testSubnetActorDiamond_Join_Works_CallRegister() public {
    //     address validator = vm.addr(1235);

    //     vm.expectCall(
    //         gatewayAddress,
    //         DEFAULT_MIN_VALIDATOR_STAKE,
    //         abi.encodeWithSelector(gwManager.register.selector),
    //         1
    //     );

    //     _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE);
    // }

    // function testSubnetActorDiamond_Join_Works_LessThanMinStake() public {
    //     address validator = vm.addr(1235);
    //     uint256 amount = DEFAULT_MIN_VALIDATOR_STAKE / 2;
    //     vm.deal(validator, amount + 1);
    //     vm.prank(validator);
    //     vm.expectCall(gatewayAddress, amount, abi.encodeWithSelector(gwManager.register.selector), 0);
    //     vm.expectCall(gatewayAddress, amount, abi.encodeWithSelector(gwManager.addStake.selector), 0);
    //     saManager.join{value: amount}(DEFAULT_NET_ADDR, FvmAddress({addrType: 1, payload: new bytes(20)}));

    //     require(saGetter.validatorCount() == 0);
    //     require(gwGetter.listSubnets().length == 0);
    // }

    // function testSubnetActorDiamond_Join_Works_MultipleNewValidators() public {
    //     _assertJoin(vm.addr(1234), DEFAULT_MIN_VALIDATOR_STAKE);
    //     _assertJoin(vm.addr(1235), DEFAULT_MIN_VALIDATOR_STAKE);

    //     require(saGetter.validatorCount() == 2);
    //     require(gwGetter.listSubnets().length == 1);
    // }

    // function testSubnetActorDiamond_Join_Works_OneValidatorWithMinimumStake() public {
    //     require(gwGetter.listSubnets().length == 0, "listSubnets correct");
    //     require(saGetter.validatorCount() == 0, "validatorCount correct");

    //     address validator = vm.addr(1234);

    //     vm.startPrank(validator);
    //     vm.deal(validator, DEFAULT_MIN_VALIDATOR_STAKE);

    //     require(validator.balance == DEFAULT_MIN_VALIDATOR_STAKE, "balance() == DEFAULT_MIN_VALIDATOR_STAKE");
    //     require(saGetter.stake(validator) == 0, "stake(validator) == 0");
    //     require(saGetter.totalStake() == 0, "totalStake() == 0");

    //     saManager.join{value: DEFAULT_MIN_VALIDATOR_STAKE}(
    //         DEFAULT_NET_ADDR,
    //         FvmAddress({addrType: 1, payload: new bytes(20)})
    //     );

    //     require(saGetter.stake(validator) == DEFAULT_MIN_VALIDATOR_STAKE);
    //     require(saGetter.totalStake() == DEFAULT_MIN_VALIDATOR_STAKE);
    //     require(validator.balance == 0);

    //     vm.stopPrank();

    //     require(saGetter.validatorCount() == 1, "validatorCount() correct");
    //     require(gwGetter.listSubnets().length == 1, "listSubnets() correct");
    // }

    // function testSubnetActorDiamond_Join_Works_NoNewValidator_CollateralNotEnough() public {
    //     address validator = vm.addr(1235);

    //     _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE - 1);

    //     require(saGetter.validatorCount() == 0);
    //     require(saGetter.status() == Status.Instantiated);
    // }

    // function testSubnetActorDiamond_Join_Works_ReactivateSubnet() public {
    //     _assertJoin(vm.addr(1234), DEFAULT_MIN_VALIDATOR_STAKE);
    //     _assertLeave(vm.addr(1234), DEFAULT_MIN_VALIDATOR_STAKE);

    //     require(saGetter.totalStake() == 0);
    //     require(saGetter.validatorCount() == 0);
    //     require(saGetter.status() == Status.Inactive);

    //     _assertJoin(vm.addr(1235), DEFAULT_MIN_VALIDATOR_STAKE);

    //     require(saGetter.validatorCount() == 1);
    //     require(saGetter.status() == Status.Active);
    // }

    // function testSubnetActorDiamond_Leave_Works_NoValidatorsLeft() public payable {
    //     address validator = address(1235);
    //     uint256 amount = DEFAULT_MIN_VALIDATOR_STAKE;

    //     _assertJoin(validator, amount);

    //     _assertLeave(validator, amount);

    //     require(saGetter.totalStake() == 0);
    //     require(saGetter.validatorCount() == 0);
    //     require(saGetter.status() == Status.Inactive);
    // }

    // function testSubnetActorDiamond_Leave_Works_StillActive() public payable {
    //     address validator1 = address(1234);
    //     address validator2 = address(1235);
    //     uint256 amount = DEFAULT_MIN_VALIDATOR_STAKE;

    //     _assertJoin(validator1, amount);
    //     _assertJoin(validator2, amount);

    //     _assertLeave(validator1, amount);

    //     require(saGetter.totalStake() == amount);
    //     require(saGetter.validatorCount() == 1);
    //     require(saGetter.status() == Status.Active);
    // }

    // function testSubnetActorDiamond_Leave_Fail_AlreadyKilled() public payable {
    //     address validator = address(1235);
    //     uint256 amount = DEFAULT_MIN_VALIDATOR_STAKE;

    //     _assertJoin(validator, amount);

    //     _assertLeave(validator, amount);
    //     _assertKill(validator);

    //     vm.prank(validator);
    //     vm.deal(validator, amount);
    //     vm.expectRevert(SubnetAlreadyKilled.selector);

    //     saManager.leave();
    // }

    // function testSubnetActorDiamond_Leave_Fail_NoStake() public payable {
    //     address caller = address(1235);

    //     vm.prank(caller);
    //     vm.deal(caller, 1 ether);

    //     vm.expectRevert(NotValidator.selector);

    //     saManager.leave();
    // }

    // Comment off first as subnet lifecycle needs update
    // function testSubnetActorDiamond_Join_Fail_AlreadyKilled() public {
    //     address validator = vm.addr(1235);

    //     _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE);
    //     _assertLeave(validator, DEFAULT_MIN_VALIDATOR_STAKE);
    //     _assertKill(validator);

    //     vm.expectRevert(SubnetAlreadyKilled.selector);
    //     vm.prank(validator);
    //     vm.deal(validator, DEFAULT_MIN_VALIDATOR_STAKE + 1);

    //     saManager.join{value: DEFAULT_MIN_VALIDATOR_STAKE}(
    //         DEFAULT_NET_ADDR,
    //         FvmAddress({addrType: 1, payload: new bytes(20)})
    //     );
    // }

    // function testSubnetActorDiamond_Kill_Works() public payable {
    //     address validator = address(1235);

    //     _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE);
    //     _assertLeave(validator, DEFAULT_MIN_VALIDATOR_STAKE);

    //     _assertKill(validator);

    //     require(gatewayAddress.balance == 0);
    //     require(gwGetter.totalSubnets() == 0);
    // }

    // function testSubnetActorDiamond_Kill_Fails_NotAllValidatorsLeft() public payable {
    //     address validator1 = address(1235);
    //     address validator2 = address(1236);

    //     _assertJoin(validator1, DEFAULT_MIN_VALIDATOR_STAKE);
    //     _assertJoin(validator2, DEFAULT_MIN_VALIDATOR_STAKE);

    //     _assertLeave(validator1, DEFAULT_MIN_VALIDATOR_STAKE);

    //     vm.prank(validator1);
    //     vm.expectRevert(NotAllValidatorsHaveLeft.selector);
    //     saManager.kill();
    // }

    // function testSubnetActorDiamond_Kill_Fails_AlreadyTerminating() public {
    //     address validator = vm.addr(1235);

    //     _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE);
    //     _assertLeave(validator, DEFAULT_MIN_VALIDATOR_STAKE);

    //     _assertKill(validator);

    //     vm.prank(validator);
    //     vm.expectRevert(SubnetAlreadyKilled.selector);

    //     saManager.kill();
    // }

    function callback() public view {
        // console.log("callback called");
    }

    // function _assertJoin(address validator, uint256 amount) internal {
    //     vm.startPrank(validator);
    //     vm.deal(validator, amount + 1);

    //     uint256 balanceBefore = validator.balance;
    //     uint256 stakeBefore = saGetter.stake(validator);
    //     uint256 totalStakeBefore = saGetter.totalStake();

    //     saManager.join{value: amount}(DEFAULT_NET_ADDR, FvmAddress({addrType: 1, payload: new bytes(20)}));

    //     require(saGetter.stake(validator) == stakeBefore + amount);
    //     require(saGetter.totalStake() == totalStakeBefore + amount);
    //     require(validator.balance == balanceBefore - amount);

    //     vm.stopPrank();
    // }

    // function _assertLeave(address validator, uint256 amount) internal {
    //     uint256 validatorBalanceBefore = validator.balance;
    //     uint256 validatorsCountBefore = saGetter.validatorCount();
    //     uint256 totalStakeBefore = saGetter.totalStake();

    //     vm.prank(validator);
    //     vm.expectCall(gatewayAddress, abi.encodeWithSelector(gwManager.releaseStake.selector, amount));
    //     vm.expectCall(validator, amount, EMPTY_BYTES);

    //     saManager.leave();

    //     require(saGetter.stake(validator) == 0);
    //     require(saGetter.totalStake() == totalStakeBefore - amount);
    //     require(saGetter.validatorCount() == validatorsCountBefore - 1);
    //     require(validator.balance == validatorBalanceBefore + amount);
    // }

    // function _assertKill(address validator) internal {
    //     vm.startPrank(validator);
    //     vm.deal(validator, 1 ether);
    //     vm.expectCall(gatewayAddress, abi.encodeWithSelector(gwManager.kill.selector));

    //     saManager.kill();

    //     require(saGetter.totalStake() == 0);
    //     require(saGetter.validatorCount() == 0);
    //     require(saGetter.status() == Status.Killed);

    //     vm.stopPrank();
    // }

    function _assertDeploySubnetActor(
        address _ipcGatewayAddr,
        ConsensusType _consensus,
        uint256 _minActivationCollateral,
        uint64 _minValidators,
        uint64 _checkPeriod,
        uint8 _majorityPercentage
    ) public {
        SubnetID memory _parentId = SubnetID(ROOTNET_CHAINID, new address[](0));

        saManager = new SubnetManagerTestUtil();
        saGetter = new SubnetActorGetterFacet();

        IDiamond.FacetCut[] memory diamondCut = new IDiamond.FacetCut[](2);

        diamondCut[0] = (
            IDiamond.FacetCut({
                facetAddress: address(saManager),
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: saManagerSelectors
            })
        );

        diamondCut[1] = (
            IDiamond.FacetCut({
                facetAddress: address(saGetter),
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: saGetterSelectors
            })
        );

        saDiamond = new SubnetActorDiamond(
            diamondCut,
            SubnetActorDiamond.ConstructorParams({
                parentId: _parentId,
                ipcGatewayAddr: _ipcGatewayAddr,
                consensus: _consensus,
                minActivationCollateral: _minActivationCollateral,
                minValidators: _minValidators,
                bottomUpCheckPeriod: _checkPeriod,
                majorityPercentage: _majorityPercentage,
                activeValidatorsLimit: 100,
                powerScale: 12,
                minCrossMsgFee: CROSS_MSG_FEE
            })
        );

        saManager = SubnetManagerTestUtil(address(saDiamond));
        saGetter = SubnetActorGetterFacet(address(saDiamond));

        require(saGetter.ipcGatewayAddr() == _ipcGatewayAddr, "saGetter.ipcGatewayAddr() == _ipcGatewayAddr");
        require(
            saGetter.minActivationCollateral() == _minActivationCollateral,
            "saGetter.minActivationCollateral() == _minActivationCollateral"
        );
        require(saGetter.minValidators() == _minValidators, "saGetter.minValidators() == _minValidators");
        require(saGetter.consensus() == _consensus);
        require(
            saGetter.getParent().toHash() == _parentId.toHash(),
            "parent.toHash() == SubnetID({root: ROOTNET_CHAINID, route: path}).toHash()"
        );
    }
}
