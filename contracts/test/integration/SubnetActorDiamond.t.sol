// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import "../../contracts/errors/IPCErrors.sol";
import {Test} from "forge-std/Test.sol";
import "forge-std/console.sol";
import {TestUtils} from "../helpers/TestUtils.sol";
import {SelectorLibrary} from "../helpers/SelectorLibrary.sol";
import {NumberContractFacetSeven} from "../helpers/contracts/NumberContractFacetSeven.sol";
import {NumberContractFacetEight} from "../helpers/contracts/NumberContractFacetEight.sol";
import {METHOD_SEND} from "../../contracts/constants/Constants.sol";
import {ConsensusType} from "../../contracts/enums/ConsensusType.sol";
import {BottomUpMsgBatch, IpcEnvelope, BottomUpCheckpoint, MAX_MSGS_PER_BATCH} from "../../contracts/structs/CrossNet.sol";
import {FvmAddress} from "../../contracts/structs/FvmAddress.sol";
import {SubnetID, PermissionMode, IPCAddress, Subnet, Asset, ValidatorInfo, AssetKind} from "../../contracts/structs/Subnet.sol";
import {IERC165} from "../../contracts/interfaces/IERC165.sol";
import {IGateway} from "../../contracts/interfaces/IGateway.sol";
import {IDiamond} from "../../contracts/interfaces/IDiamond.sol";
import {IDiamondCut} from "../../contracts/interfaces/IDiamondCut.sol";
import {IDiamondLoupe} from "../../contracts/interfaces/IDiamondLoupe.sol";
import {FvmAddressHelper} from "../../contracts/lib/FvmAddressHelper.sol";
import {MultisignatureChecker} from "../../contracts/lib/LibMultisignatureChecker.sol";
import {SubnetIDHelper} from "../../contracts/lib/SubnetIDHelper.sol";
import {GatewayDiamond} from "../../contracts/GatewayDiamond.sol";
import {SubnetActorDiamond, FunctionNotFound} from "../../contracts/SubnetActorDiamond.sol";
import {SubnetActorManagerFacet} from "../../contracts/subnet/SubnetActorManagerFacet.sol";
import {OwnershipFacet} from "../../contracts/OwnershipFacet.sol";
import {SubnetActorGetterFacet} from "../../contracts/subnet/SubnetActorGetterFacet.sol";
import {SubnetActorPauseFacet} from "../../contracts/subnet/SubnetActorPauseFacet.sol";
import {SubnetActorCheckpointingFacet} from "../../contracts/subnet/SubnetActorCheckpointingFacet.sol";
import {SubnetActorRewardFacet} from "../../contracts/subnet/SubnetActorRewardFacet.sol";
import {DiamondCutFacet} from "../../contracts/diamond/DiamondCutFacet.sol";
import {FilAddress} from "fevmate/contracts/utils/FilAddress.sol";
import {LibStaking} from "../../contracts/lib/LibStaking.sol";
import {LibDiamond} from "../../contracts/lib/LibDiamond.sol";
import {Pausable} from "../../contracts/lib/LibPausable.sol";
import {AssetHelper} from "../../contracts/lib/AssetHelper.sol";

import {IntegrationTestBase} from "../IntegrationTestBase.sol";

import {SubnetActorFacetsHelper} from "../helpers/SubnetActorFacetsHelper.sol";
import {GatewayFacetsHelper} from "../helpers/GatewayFacetsHelper.sol";
import {ERC20PresetFixedSupply} from "../helpers/ERC20PresetFixedSupply.sol";
import {SubnetValidatorGater} from "../../contracts/examples/SubnetValidatorGater.sol";

import {FullActivityRollup, Consensus} from "../../contracts/structs/Activity.sol";
import {ValidatorRewarderMap} from "../../contracts/examples/ValidatorRewarderMap.sol";
import {MintingValidatorRewarder} from "../../contracts/examples/MintingValidatorRewarder.sol";
import {MerkleTreeHelper} from "../helpers/MerkleTreeHelper.sol";
import {ActivityHelper} from "../helpers/ActivityHelper.sol";

contract SubnetActorDiamondTest is Test, IntegrationTestBase {
    using SubnetIDHelper for SubnetID;
    using FilAddress for address;
    using FvmAddressHelper for FvmAddress;
    using SubnetActorFacetsHelper for SubnetActorDiamond;
    using GatewayFacetsHelper for GatewayDiamond;

    address gatewayAddress;

    function setUp() public override {
        super.setUp();

        gatewayAddress = address(gatewayDiamond);
    }

    function testSubnetActorDiamond_NewSubnetActorWithDefaultParams() public view {
        SubnetID memory _parentId = SubnetID(ROOTNET_CHAINID, new address[](0));
        SubnetActorDiamond.ConstructorParams memory params = defaultSubnetActorParamsWith(address(gatewayDiamond));

        require(saDiamond.getter().ipcGatewayAddr() == params.ipcGatewayAddr, "unexpected gateway");
        require(
            saDiamond.getter().minActivationCollateral() == params.minActivationCollateral,
            "unexpected collateral"
        );
        require(saDiamond.getter().minValidators() == params.minValidators, "unexpected minValidators");
        require(saDiamond.getter().consensus() == params.consensus, "unexpected consensus");
        require(saDiamond.getter().getParent().equals(_parentId), "unexpected parent");
        require(saDiamond.getter().activeValidatorsLimit() == 100, "unexpected activeValidatorsLimit");
        require(saDiamond.getter().powerScale() == params.powerScale, "unexpected powerscale");
        require(saDiamond.getter().bottomUpCheckPeriod() == params.bottomUpCheckPeriod, "unexpected bottom-up period");
        require(saDiamond.getter().majorityPercentage() == params.majorityPercentage, "unexpected majority percentage");
        require(saDiamond.getter().getParent().toHash() == _parentId.toHash(), "unexpected parent subnetID hash");
        require(saDiamond.getter().genesisValidators().length == 0, "unexpected genesis validators");
        require(saDiamond.getter().getActiveValidators().length == 0, "unexpected active validators");
        require(saDiamond.getter().getWaitingValidators().length == 0, "unexpected waiting validators");
    }

    function testSubnetActorDiamondReal_LoupeFunction() public view {
        require(saDiamond.diamondLouper().facets().length == 9, "unexpected length");
        require(
            saDiamond.diamondLouper().supportsInterface(type(IERC165).interfaceId) == true,
            "IERC165 not supported"
        );
        require(
            saDiamond.diamondLouper().supportsInterface(type(IDiamondCut).interfaceId) == true,
            "IDiamondCut not supported"
        );
        require(
            saDiamond.diamondLouper().supportsInterface(type(IDiamondLoupe).interfaceId) == true,
            "IDiamondLoupe not supported"
        );
    }

    /// @notice Testing the basic join, stake, leave lifecycle of validators
    function testSubnetActorDiamond_BasicLifeCycle() public {
        (address validator1, uint256 privKey1, bytes memory publicKey1) = TestUtils.newValidator(100);
        (address validator2, uint256 privKey2, bytes memory publicKey2) = TestUtils.newValidator(101);

        // total collateral in the gateway
        uint256 collateral = 0;
        uint256 stake = 10;
        uint256 validator1Stake = 10 * DEFAULT_MIN_VALIDATOR_STAKE;

        require(!saDiamond.getter().isActiveValidator(validator1), "active validator1");
        require(!saDiamond.getter().isWaitingValidator(validator1), "waiting validator1");

        require(!saDiamond.getter().isActiveValidator(validator2), "active validator2");
        require(!saDiamond.getter().isWaitingValidator(validator2), "waiting validator2");

        // ======== Step. Join ======
        // initial validator joins
        vm.deal(validator1, validator1Stake);
        vm.deal(validator2, DEFAULT_MIN_VALIDATOR_STAKE);

        vm.startPrank(validator1);
        saDiamond.manager().join{value: validator1Stake}(publicKey1, validator1Stake);
        collateral = validator1Stake;

        require(gatewayAddress.balance == collateral, "gw balance is incorrect after validator1 joining");

        // collateral confirmed immediately and network boostrapped
        ValidatorInfo memory v = saDiamond.getter().getValidator(validator1);
        require(v.totalCollateral == collateral, "total collateral not expected");
        require(v.confirmedCollateral == collateral, "confirmed collateral not equal to collateral");
        require(saDiamond.getter().isActiveValidator(validator1), "not active validator 1");
        require(!saDiamond.getter().isWaitingValidator(validator1), "waiting validator 1");
        require(!saDiamond.getter().isActiveValidator(validator2), "active validator2");
        require(!saDiamond.getter().isWaitingValidator(validator2), "waiting validator2");
        TestUtils.ensureBytesEqual(v.metadata, publicKey1);
        require(saDiamond.getter().bootstrapped(), "subnet not bootstrapped");
        require(!saDiamond.getter().killed(), "subnet killed");
        require(saDiamond.getter().genesisValidators().length == 1, "not 1 genesis validator");
        require(saDiamond.getter().getActiveValidators().length == 1, "not 1 active validator");
        require(saDiamond.getter().getWaitingValidators().length == 0, "not 0 waiting validator");

        (uint64 nextConfigNum, uint64 startConfigNum) = saDiamond.getter().getConfigurationNumbers();
        require(nextConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER, "next config num not 1");
        require(startConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER, "start config num not 1");

        vm.stopPrank();

        // subnet bootstrapped and should go through queue
        // second and third validators join
        vm.startPrank(validator2);
        saDiamond.manager().join{value: DEFAULT_MIN_VALIDATOR_STAKE}(publicKey2, DEFAULT_MIN_VALIDATOR_STAKE);

        // collateral not confirmed yet
        v = saDiamond.getter().getValidator(validator2);
        require(gatewayAddress.balance == collateral, "gw balance is incorrect after validator2 joining");
        require(v.totalCollateral == DEFAULT_MIN_VALIDATOR_STAKE, "total collateral not expected");
        require(v.confirmedCollateral == 0, "confirmed collateral not equal to collateral");
        require(saDiamond.getter().isActiveValidator(validator1), "not active validator 1");
        require(!saDiamond.getter().isWaitingValidator(validator1), "waiting validator 1");
        require(!saDiamond.getter().isActiveValidator(validator2), "active validator2");
        require(!saDiamond.getter().isWaitingValidator(validator2), "waiting validator2");
        TestUtils.ensureBytesEqual(v.metadata, new bytes(0));

        (nextConfigNum, startConfigNum) = saDiamond.getter().getConfigurationNumbers();
        // join will update the metadata, incr by 1
        // join will call deposit, incr by 1
        require(nextConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER + 2, "next config num not 3");
        require(startConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER, "start config num not 1");
        vm.stopPrank();

        // ======== Step. Confirm join operation ======
        collateral += DEFAULT_MIN_VALIDATOR_STAKE;
        confirmChange(validator1, privKey1);
        require(gatewayAddress.balance == collateral, "gw balance is incorrect after validator2 joining");

        v = saDiamond.getter().getValidator(validator2);
        require(v.totalCollateral == DEFAULT_MIN_VALIDATOR_STAKE, "unexpected total collateral after confirm join");
        require(
            v.confirmedCollateral == DEFAULT_MIN_VALIDATOR_STAKE,
            "unexpected confirmed collateral after confirm join"
        );
        require(saDiamond.getter().isActiveValidator(validator1), "not active validator1");
        require(!saDiamond.getter().isWaitingValidator(validator1), "waiting validator1");
        require(saDiamond.getter().isActiveValidator(validator2), "not active validator2");
        require(!saDiamond.getter().isWaitingValidator(validator2), "waiting validator2");
        require(saDiamond.getter().getActiveValidators().length == 2, "not 2 active validators");
        require(saDiamond.getter().getWaitingValidators().length == 0, "not 0 waiting validators");

        (nextConfigNum, startConfigNum) = saDiamond.getter().getConfigurationNumbers();
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
        vm.deal(validator1, stake);

        saDiamond.manager().stake{value: stake}(stake);

        v = saDiamond.getter().getValidator(validator1);
        require(v.totalCollateral == validator1Stake + stake, "unexpected total collateral after stake");
        require(v.confirmedCollateral == validator1Stake, "unexpected confirmed collateral after stake");
        require(gatewayAddress.balance == collateral, "gw balance is incorrect after validator1 stakes more");

        (nextConfigNum, startConfigNum) = saDiamond.getter().getConfigurationNumbers();
        require(nextConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER + 3, "next config num not 4 after stake");
        require(startConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER + 2, "start config num not 3 after stake");

        vm.stopPrank();

        // ======== Step. Confirm stake operation ======
        collateral += stake;
        confirmChange(validator1, privKey1, validator2, privKey2);
        require(gatewayAddress.balance == collateral, "gw balance is incorrect after confirm stake");

        v = saDiamond.getter().getValidator(validator1);
        require(v.totalCollateral == validator1Stake + stake, "unexpected total collateral after confirm stake");
        require(
            v.confirmedCollateral == validator1Stake + stake,
            "unexpected confirmed collateral after confirm stake"
        );

        v = saDiamond.getter().getValidator(validator2);
        require(v.totalCollateral == DEFAULT_MIN_VALIDATOR_STAKE, "unexpected total collateral after confirm stake");
        require(
            v.confirmedCollateral == DEFAULT_MIN_VALIDATOR_STAKE,
            "unexpected confirmed collateral after confirm stake"
        );

        (nextConfigNum, startConfigNum) = saDiamond.getter().getConfigurationNumbers();
        require(
            nextConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER + 3,
            "next config num not 4 after confirm stake"
        );
        require(
            startConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER + 3,
            "start config num not 4 after confirm stake"
        );
        require(saDiamond.getter().genesisValidators().length == 1, "genesis validators still 1");

        // ======== Step. Leave ======
        vm.startPrank(validator1);
        saDiamond.manager().leave();

        v = saDiamond.getter().getValidator(validator1);
        require(v.totalCollateral == 0, "total collateral not 0 after confirm leave");
        require(v.confirmedCollateral == validator1Stake + stake, "confirmed collateral incorrect after confirm leave");
        require(gatewayAddress.balance == collateral, "gw balance is incorrect after validator 1 leaving");

        (nextConfigNum, startConfigNum) = saDiamond.getter().getConfigurationNumbers();
        require(
            nextConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER + 4,
            "next config num not 5 after confirm leave"
        );
        require(
            startConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER + 3,
            "start config num not 4 after confirm leave"
        );
        require(saDiamond.getter().isActiveValidator(validator1), "not active validator 1");
        require(saDiamond.getter().isActiveValidator(validator2), "not active validator 2");

        vm.stopPrank();

        // ======== Step. Confirm leave ======
        confirmChange(validator1, privKey1);
        collateral -= (validator1Stake + stake);
        require(gatewayAddress.balance == collateral, "gw balance is incorrect after confirming validator 1 leaving");

        v = saDiamond.getter().getValidator(validator1);
        require(v.totalCollateral == 0, "total collateral not 0 after confirm leave");
        require(v.confirmedCollateral == 0, "confirmed collateral not 0 after confirm leave");

        (nextConfigNum, startConfigNum) = saDiamond.getter().getConfigurationNumbers();
        require(
            nextConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER + 4,
            "next config num not 5 after confirm leave"
        );
        require(
            startConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER + 4,
            "start config num not 5 after confirm leave"
        );
        require(!saDiamond.getter().isActiveValidator(validator1), "active validator 1");
        require(saDiamond.getter().isActiveValidator(validator2), "not active validator 2");

        // ======== Step. Claim collateral ======
        uint256 b1 = validator1.balance;
        vm.prank(validator1);
        saDiamond.rewarder().claim();
        uint256 b2 = validator1.balance;
        require(b2 - b1 == validator1Stake + stake, "collateral not received");
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

        createSubnetActor(
            _ipcGatewayAddr,
            ConsensusType.Fendermint,
            _minActivationCollateral,
            _minValidators,
            _checkPeriod,
            _majorityPercentage
        );

        SubnetID memory parent = saDiamond.getter().getParent();
        require(parent.isRoot(), "parent.isRoot()");

        require(saDiamond.getter().bottomUpCheckPeriod() == _checkPeriod, "bottomUpCheckPeriod");
    }

    function testSubnetActorDiamond_Deployments_Fail_GatewayCannotBeZero() public {
        SubnetActorManagerFacet saDupMangerFaucet = new SubnetActorManagerFacet();
        SubnetActorGetterFacet saDupGetterFaucet = new SubnetActorGetterFacet();
        SubnetActorPauseFacet saDupPauserFaucet = new SubnetActorPauseFacet();
        SubnetActorRewardFacet saDupRewardFaucet = new SubnetActorRewardFacet();
        SubnetActorCheckpointingFacet saDupCheckpointerFaucet = new SubnetActorCheckpointingFacet();
        OwnershipFacet saOwnershipFacet = new OwnershipFacet();

        Asset memory native = AssetHelper.native();

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
                permissionMode: PermissionMode.Collateral,
                supplySource: native,
                collateralSource: AssetHelper.native(),
                validatorGater: address(0),
                validatorRewarder: address(0)
            }),
            address(saDupGetterFaucet),
            address(saDupMangerFaucet),
            address(saDupPauserFaucet),
            address(saDupRewardFaucet),
            address(saDupCheckpointerFaucet),
            address(saOwnershipFacet)
        );
    }

    function testSubnetActorDiamond_Join_Fail_NotOwnerOfPublicKey() public {
        address validator = vm.addr(100);

        vm.deal(validator, 1 gwei);
        vm.prank(validator);
        vm.expectRevert(NotOwnerOfPublicKey.selector);

        saDiamond.manager().join{value: 10}(new bytes(65), 10);
    }

    function testSubnetActorDiamond_Join_Fail_InvalidPublicKeyLength() public {
        address validator = vm.addr(100);

        vm.deal(validator, 1 gwei);
        vm.prank(validator);
        vm.expectRevert(InvalidPublicKeyLength.selector);

        saDiamond.manager().join{value: 10}(new bytes(64), 10);
    }

    function testSubnetActorDiamond_Join_Fail_ZeroColalteral() public {
        (address validator, bytes memory publicKey) = TestUtils.deriveValidatorAddress(100);

        vm.deal(validator, 1 gwei);
        vm.prank(validator);
        vm.expectRevert(CollateralIsZero.selector);

        saDiamond.manager().join(publicKey, 0);
    }

    function testSubnetActorDiamond_Bootstrap_Node() public {
        (address validator, uint256 privKey, bytes memory publicKey) = TestUtils.newValidator(100);

        vm.deal(validator, DEFAULT_MIN_VALIDATOR_STAKE);
        vm.prank(validator);
        saDiamond.manager().join{value: DEFAULT_MIN_VALIDATOR_STAKE}(publicKey, DEFAULT_MIN_VALIDATOR_STAKE);

        // validator adds empty node
        vm.prank(validator);
        vm.expectRevert(EmptyAddress.selector);
        saDiamond.manager().addBootstrapNode("");

        // validator adds a node
        vm.prank(validator);
        saDiamond.manager().addBootstrapNode("1.2.3.4");

        // not-validator adds a node
        vm.prank(vm.addr(200));
        vm.expectRevert(abi.encodeWithSelector(NotValidator.selector, vm.addr(200)));
        saDiamond.manager().addBootstrapNode("3.4.5.6");

        string[] memory nodes = saDiamond.getter().getBootstrapNodes();
        require(nodes.length == 1, "it returns one node");
        require(
            keccak256(abi.encodePacked((nodes[0]))) == keccak256(abi.encodePacked(("1.2.3.4"))),
            "it returns correct address"
        );

        vm.prank(validator);
        saDiamond.manager().leave();
        confirmChange(validator, privKey);

        nodes = saDiamond.getter().getBootstrapNodes();
        require(nodes.length == 0, "no nodes");
    }

    function testSubnetActorDiamond_Leave_NotValidator() public {
        (address validator, , ) = TestUtils.newValidator(100);

        // non-empty subnet can't be killed
        vm.prank(validator);
        vm.expectRevert(abi.encodeWithSelector(NotValidator.selector, validator));
        saDiamond.manager().leave();
    }

    function testSubnetActorDiamond_Leave_Subnet() public {
        (address validator1, uint256 privKey1, bytes memory publicKey1) = TestUtils.newValidator(100);
        (address validator2, uint256 privKey2, bytes memory publicKey2) = TestUtils.newValidator(101);
        (address validator3, uint256 privKey3, bytes memory publicKey3) = TestUtils.newValidator(102);

        vm.deal(validator1, DEFAULT_MIN_VALIDATOR_STAKE);
        vm.deal(validator2, 3 * DEFAULT_MIN_VALIDATOR_STAKE);
        vm.deal(validator3, DEFAULT_MIN_VALIDATOR_STAKE);

        vm.prank(validator1);
        saDiamond.manager().join{value: DEFAULT_MIN_VALIDATOR_STAKE}(publicKey1, DEFAULT_MIN_VALIDATOR_STAKE);

        vm.prank(validator2);
        saDiamond.manager().join{value: 3 * DEFAULT_MIN_VALIDATOR_STAKE}(publicKey2, 3 * DEFAULT_MIN_VALIDATOR_STAKE);

        vm.prank(validator3);
        saDiamond.manager().join{value: DEFAULT_MIN_VALIDATOR_STAKE}(publicKey3, DEFAULT_MIN_VALIDATOR_STAKE);

        confirmChange(validator1, privKey1);

        require(saDiamond.getter().isActiveValidator(validator1), "validator 1 is not active");
        require(saDiamond.getter().isActiveValidator(validator2), "validator 2 is not active");
        require(saDiamond.getter().isActiveValidator(validator3), "validator 3 is not active");

        // non-empty subnet can't be killed
        vm.expectRevert(NotAllValidatorsHaveLeft.selector);
        vm.prank(validator1);
        saDiamond.manager().kill();

        // validator1 is leaving the subnet
        vm.startPrank(validator1);
        saDiamond.manager().leave();
        vm.stopPrank();

        confirmChange(validator2, privKey2, validator3, privKey3);

        require(!saDiamond.getter().isActiveValidator(validator1), "validator 1 is active");
        require(saDiamond.getter().isActiveValidator(validator2), "validator 2 is not active");
        require(saDiamond.getter().isActiveValidator(validator3), "validator 3 is not active");
    }

    function testSubnetActorDiamond_Kill_NotBootstrappedSubnet() public {
        (address validator1, , ) = TestUtils.newValidator(100);

        // not bootstrapped subnet can't be killed
        vm.expectRevert(SubnetNotBootstrapped.selector);
        vm.prank(validator1);
        saDiamond.manager().kill();
    }

    function testSubnetActorDiamond_Stake() public {
        (address validator, bytes memory publicKey) = TestUtils.deriveValidatorAddress(100);
        vm.deal(validator, 10 gwei);

        vm.prank(validator);
        vm.expectRevert(CollateralIsZero.selector);
        saDiamond.manager().stake(0);

        vm.prank(validator);
        vm.expectRevert((abi.encodeWithSelector(MethodNotAllowed.selector, ERR_VALIDATOR_NOT_JOINED)));
        saDiamond.manager().stake{value: 10}(10);

        vm.prank(validator);
        saDiamond.manager().join{value: 3}(publicKey, 3);

        ValidatorInfo memory info = saDiamond.getter().getValidator(validator);
        require(info.totalCollateral == 3);
    }

    function testSubnetActorDiamond_crossMsgGetter() public view {
        IpcEnvelope[] memory msgs = new IpcEnvelope[](1);
        msgs[0] = TestUtils.newXnetCallMsg(
            IPCAddress({subnetId: saDiamond.getter().getParent(), rawAddress: FvmAddressHelper.from(address(this))}),
            IPCAddress({subnetId: saDiamond.getter().getParent(), rawAddress: FvmAddressHelper.from(address(this))}),
            DEFAULT_CROSS_MSG_FEE + 1,
            0
        );
        require(saDiamond.getter().crossMsgsHash(msgs) == keccak256(abi.encode(msgs)));
    }

    function testSubnetActorDiamond_validateActiveQuorumSignatures() public {
        (uint256[] memory keys, address[] memory validators, ) = TestUtils.getThreeValidators(vm);

        bytes[] memory pubKeys = new bytes[](3);
        bytes[] memory signatures = new bytes[](3);

        bytes32 hash = keccak256(abi.encodePacked("test"));

        for (uint256 i = 0; i < 3; i++) {
            (uint8 v, bytes32 r, bytes32 s) = vm.sign(keys[i], hash);
            signatures[i] = abi.encodePacked(r, s, v);
            pubKeys[i] = TestUtils.deriveValidatorPubKeyBytes(keys[i]);
            vm.deal(validators[i], 10 gwei);
            vm.prank(validators[i]);
            saDiamond.manager().join{value: 10}(pubKeys[i], 10);
        }

        saDiamond.checkpointer().validateActiveQuorumSignatures(validators, hash, signatures);
    }

    function testSubnetActorDiamond_validateActiveQuorumSignatures_InvalidWeightSum() public {
        (uint256[] memory keys, address[] memory validators, ) = TestUtils.getThreeValidators(vm);

        bytes[] memory pubKeys = new bytes[](3);
        bytes[] memory signatures = new bytes[](1);
        address[] memory subValidators = new address[](1);

        bytes32 hash = keccak256(abi.encodePacked("test"));

        for (uint256 i = 0; i < 3; i++) {
            pubKeys[i] = TestUtils.deriveValidatorPubKeyBytes(keys[i]);
            vm.deal(validators[i], 10 gwei);
            vm.prank(validators[i]);
            saDiamond.manager().join{value: 10}(pubKeys[i], 10);
        }

        // this should trigger `WeightsSumLessThanThreshold` error since the signature weight will be just 100.
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(keys[0], hash);
        signatures[0] = abi.encodePacked(r, s, v);
        subValidators[0] = validators[0];

        vm.expectRevert(
            abi.encodeWithSelector(
                InvalidSignatureErr.selector,
                MultisignatureChecker.Error.WeightsSumLessThanThreshold
            )
        );
        saDiamond.checkpointer().validateActiveQuorumSignatures(subValidators, hash, signatures);
    }

    function testSubnetActorDiamond_validateActiveQuorumSignatures_InvalidSignature() public {
        (uint256[] memory keys, address[] memory validators, ) = TestUtils.getThreeValidators(vm);
        bytes[] memory pubKeys = new bytes[](3);
        bytes[] memory signatures = new bytes[](3);

        bytes32 hash = keccak256(abi.encodePacked("test"));

        uint8 vv = 255;

        for (uint256 i = 0; i < 3; i++) {
            (, bytes32 r, bytes32 s) = vm.sign(keys[i], hash);

            // create incorrect signature using `vv` to trigger `InvalidSignature` error.
            signatures[i] = abi.encodePacked(r, s, vv);

            pubKeys[i] = TestUtils.deriveValidatorPubKeyBytes(keys[i]);
            vm.deal(validators[i], 10 gwei);
            vm.prank(validators[i]);
            saDiamond.manager().join{value: 10}(pubKeys[i], 10);
        }

        vm.expectRevert(
            abi.encodeWithSelector(InvalidSignatureErr.selector, MultisignatureChecker.Error.InvalidSignature)
        );
        saDiamond.checkpointer().validateActiveQuorumSignatures(validators, hash, signatures);
    }

    function testSubnetActorDiamond_validateActiveQuorumSignatures_EmptySignatures() public {
        (uint256[] memory keys, address[] memory validators, ) = TestUtils.getThreeValidators(vm);
        bytes[] memory pubKeys = new bytes[](3);
        bytes[] memory signatures = new bytes[](0);

        bytes32 hash = keccak256(abi.encodePacked("test"));

        for (uint256 i = 0; i < 3; i++) {
            pubKeys[i] = TestUtils.deriveValidatorPubKeyBytes(keys[i]);
            vm.deal(validators[i], 10 gwei);
            vm.prank(validators[i]);
            saDiamond.manager().join{value: 10}(pubKeys[i], 10);
        }

        require(signatures.length == 0, "signatures are not empty");
        vm.expectRevert(
            abi.encodeWithSelector(InvalidSignatureErr.selector, MultisignatureChecker.Error.EmptySignatures)
        );
        saDiamond.checkpointer().validateActiveQuorumSignatures(validators, hash, signatures);
    }

    function testSubnetActorDiamond_validateActiveQuorumSignatures_InvalidArrayLength() public {
        (uint256[] memory keys, address[] memory validators, ) = TestUtils.getThreeValidators(vm);
        bytes[] memory pubKeys = new bytes[](3);
        bytes[] memory signatures = new bytes[](1);

        bytes32 hash = keccak256(abi.encodePacked("test"));

        for (uint256 i = 0; i < 3; i++) {
            pubKeys[i] = TestUtils.deriveValidatorPubKeyBytes(keys[i]);
            vm.deal(validators[i], 10 gwei);
            vm.prank(validators[i]);
            saDiamond.manager().join{value: 10}(pubKeys[i], 10);
        }

        require(signatures.length == 1, "signatures are not empty");
        vm.expectRevert(
            abi.encodeWithSelector(InvalidSignatureErr.selector, MultisignatureChecker.Error.InvalidArrayLength)
        );
        saDiamond.checkpointer().validateActiveQuorumSignatures(validators, hash, signatures);
    }

    function testSubnetActorDiamond_validateActiveQuorumSignatures_InvalidSignatory() public {
        (uint256[] memory keys, address[] memory validators, ) = TestUtils.getThreeValidators(vm);
        bytes[] memory pubKeys = new bytes[](3);
        bytes[] memory signatures = new bytes[](3);

        bytes32 hash = keccak256(abi.encodePacked("test"));
        bytes32 hash0 = keccak256(abi.encodePacked("test1"));

        for (uint256 i = 0; i < 3; i++) {
            (uint8 v, bytes32 r, bytes32 s) = vm.sign(keys[i], hash);

            // create incorrect signature using `vv`
            signatures[i] = abi.encodePacked(r, s, v);

            pubKeys[i] = TestUtils.deriveValidatorPubKeyBytes(keys[i]);
            vm.deal(validators[i], 10 gwei);
            vm.prank(validators[i]);
            saDiamond.manager().join{value: 10}(pubKeys[i], 10);
        }

        // swap validators to trigger `InvalidSignatory` error;
        address a;
        a = validators[0];
        validators[0] = validators[1];
        validators[1] = a;

        vm.expectRevert(
            abi.encodeWithSelector(InvalidSignatureErr.selector, MultisignatureChecker.Error.InvalidSignatory)
        );
        saDiamond.checkpointer().validateActiveQuorumSignatures(validators, hash0, signatures);
    }

    function testSubnetActorDiamond_submitCheckpoint_basic() public {
        (uint256[] memory keys, address[] memory validators, ) = TestUtils.getThreeValidators(vm);
        bytes[] memory pubKeys = new bytes[](3);
        bytes[] memory signatures = new bytes[](3);

        for (uint256 i = 0; i < 3; i++) {
            vm.deal(validators[i], 10 gwei);
            pubKeys[i] = TestUtils.deriveValidatorPubKeyBytes(keys[i]);
            vm.prank(validators[i]);
            saDiamond.manager().join{value: 10}(pubKeys[i], 10);
        }

        SubnetID memory localSubnetID = saDiamond.getter().getParent().createSubnetId(address(saDiamond));

        IpcEnvelope memory crossMsg = TestUtils.newXnetCallMsg(
            IPCAddress({subnetId: localSubnetID, rawAddress: FvmAddressHelper.from(address(saDiamond))}),
            IPCAddress({
                subnetId: saDiamond.getter().getParent(),
                rawAddress: FvmAddressHelper.from(address(saDiamond))
            }),
            DEFAULT_CROSS_MSG_FEE + 1,
            0
        );
        IpcEnvelope[] memory msgs = new IpcEnvelope[](1);
        msgs[0] = crossMsg;

        BottomUpCheckpoint memory checkpoint = BottomUpCheckpoint({
            subnetID: localSubnetID,
            blockHeight: saDiamond.getter().bottomUpCheckPeriod(),
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 0,
            msgs: msgs,
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });

        BottomUpCheckpoint memory checkpointWithIncorrectHeight = BottomUpCheckpoint({
            subnetID: saDiamond.getter().getParent(),
            blockHeight: 1,
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 0,
            msgs: msgs,
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });

        vm.deal(address(saDiamond), 100 ether);
        vm.prank(address(saDiamond));
        gatewayDiamond.manager().register{value: DEFAULT_MIN_VALIDATOR_STAKE + 3 * DEFAULT_CROSS_MSG_FEE}(
            3 * DEFAULT_CROSS_MSG_FEE,
            DEFAULT_MIN_VALIDATOR_STAKE
        );

        bytes32 hash = keccak256(abi.encode(checkpoint));

        for (uint256 i = 0; i < 3; i++) {
            (uint8 v, bytes32 r, bytes32 s) = vm.sign(keys[i], hash);
            signatures[i] = abi.encodePacked(r, s, v);
        }

        vm.expectRevert(InvalidCheckpointEpoch.selector);
        vm.prank(validators[0]);
        saDiamond.checkpointer().submitCheckpoint(checkpointWithIncorrectHeight, validators, signatures);

        // skip the current checkpoint, should fail
        checkpointWithIncorrectHeight.blockHeight = saDiamond.getter().bottomUpCheckPeriod() + 1;
        vm.expectRevert(CannotSubmitFutureCheckpoint.selector);
        vm.prank(validators[0]);
        saDiamond.checkpointer().submitCheckpoint(checkpointWithIncorrectHeight, validators, signatures);

        // skip the curent checkpoint but submit at the next bottom up checkpoint, should fail
        checkpointWithIncorrectHeight.blockHeight = saDiamond.getter().bottomUpCheckPeriod() * 2;
        vm.expectRevert(CannotSubmitFutureCheckpoint.selector);
        vm.prank(validators[0]);
        saDiamond.checkpointer().submitCheckpoint(checkpointWithIncorrectHeight, validators, signatures);

        vm.expectCall(gatewayAddress, abi.encodeCall(IGateway.commitCheckpoint, (checkpoint)), 1);
        vm.prank(validators[0]);
        saDiamond.checkpointer().submitCheckpoint(checkpoint, validators, signatures);

        require(
            saDiamond.getter().lastBottomUpCheckpointHeight() == saDiamond.getter().bottomUpCheckPeriod(),
            " checkpoint height correct"
        );

        vm.expectRevert(BottomUpCheckpointAlreadySubmitted.selector);
        vm.prank(validators[0]);
        saDiamond.checkpointer().submitCheckpoint(checkpoint, validators, signatures);
        require(
            saDiamond.getter().lastBottomUpCheckpointHeight() == saDiamond.getter().bottomUpCheckPeriod(),
            " checkpoint height correct"
        );

        (bool exists, BottomUpCheckpoint memory recvCheckpoint) = saDiamond.getter().bottomUpCheckpointAtEpoch(
            saDiamond.getter().bottomUpCheckPeriod()
        );
        require(exists, "checkpoint does not exist");
        require(hash == keccak256(abi.encode(recvCheckpoint)), "checkpoint hashes are not the same");

        bytes32 recvHash;
        (exists, recvHash) = saDiamond.getter().bottomUpCheckpointHashAtEpoch(saDiamond.getter().bottomUpCheckPeriod());
        require(exists, "checkpoint does not exist");
        require(hash == recvHash, "hashes are not the same");

        saDiamond.pauser().pause();
        vm.prank(validators[0]);
        vm.expectRevert(Pausable.EnforcedPause.selector);
        saDiamond.checkpointer().submitCheckpoint(checkpoint, validators, signatures);
    }

    function testSubnetActorDiamond_submitCheckpoint_msgBatchFull() public {
        (uint256[] memory keys, address[] memory validators, ) = TestUtils.getThreeValidators(vm);
        bytes[] memory pubKeys = new bytes[](3);
        bytes[] memory signatures = new bytes[](3);

        for (uint256 i = 0; i < 3; i++) {
            vm.deal(validators[i], 10 gwei);
            pubKeys[i] = TestUtils.deriveValidatorPubKeyBytes(keys[i]);
            vm.prank(validators[i]);
            saDiamond.manager().join{value: 10}(pubKeys[i], 10);
        }

        SubnetID memory localSubnetID = saDiamond.getter().getParent().createSubnetId(address(saDiamond));

        IpcEnvelope[] memory msgs = new IpcEnvelope[](MAX_MSGS_PER_BATCH);
        for (uint256 i = 0; i < MAX_MSGS_PER_BATCH; i++) {
            IpcEnvelope memory crossMsg = TestUtils.newXnetCallMsg(
                IPCAddress({subnetId: localSubnetID, rawAddress: FvmAddressHelper.from(address(saDiamond))}),
                IPCAddress({
                    subnetId: saDiamond.getter().getParent(),
                    rawAddress: FvmAddressHelper.from(address(saDiamond))
                }),
                1,
                uint64(i)
            );
            msgs[i] = crossMsg;
        }

        BottomUpCheckpoint memory checkpoint = BottomUpCheckpoint({
            subnetID: localSubnetID,
            blockHeight: 1,
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 0,
            msgs: msgs,
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });

        BottomUpCheckpoint memory checkpointWithIncorrectHeight = BottomUpCheckpoint({
            subnetID: saDiamond.getter().getParent(),
            blockHeight: 1,
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 0,
            msgs: new IpcEnvelope[](0),
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });

        vm.deal(address(saDiamond), 100 ether);
        vm.prank(address(saDiamond));
        gatewayDiamond.manager().register{value: DEFAULT_MIN_VALIDATOR_STAKE + 3 * DEFAULT_CROSS_MSG_FEE}(
            3 * DEFAULT_CROSS_MSG_FEE,
            DEFAULT_MIN_VALIDATOR_STAKE
        );

        bytes32 hash = keccak256(abi.encode(checkpoint));

        for (uint256 i = 0; i < 3; i++) {
            (uint8 v, bytes32 r, bytes32 s) = vm.sign(keys[i], hash);
            signatures[i] = abi.encodePacked(r, s, v);
        }

        vm.expectRevert(InvalidCheckpointEpoch.selector);
        vm.prank(validators[0]);
        saDiamond.checkpointer().submitCheckpoint(checkpointWithIncorrectHeight, validators, signatures);

        vm.expectCall(gatewayAddress, abi.encodeCall(IGateway.commitCheckpoint, (checkpoint)), 1);
        vm.prank(validators[0]);
        saDiamond.checkpointer().submitCheckpoint(checkpoint, validators, signatures);

        require(saDiamond.getter().lastBottomUpCheckpointHeight() == 1, " checkpoint height correct");

        // submit another again
        checkpoint.blockHeight = 2;
        checkpoint.activity = ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)));
        hash = keccak256(abi.encode(checkpoint));

        for (uint256 i = 0; i < 3; i++) {
            (uint8 v, bytes32 r, bytes32 s) = vm.sign(keys[i], hash);
            signatures[i] = abi.encodePacked(r, s, v);
        }

        vm.expectCall(gatewayAddress, abi.encodeCall(IGateway.commitCheckpoint, (checkpoint)), 1);
        vm.prank(validators[0]);
        saDiamond.checkpointer().submitCheckpoint(checkpoint, validators, signatures);
    }

    function testSubnetActorDiamond_submitCheckpoint_mixAndMatch() public {
        (uint256[] memory keys, address[] memory validators, ) = TestUtils.getThreeValidators(vm);
        bytes[] memory pubKeys = new bytes[](3);
        bytes[] memory signatures = new bytes[](3);

        for (uint256 i = 0; i < 3; i++) {
            vm.deal(validators[i], 10 gwei);
            pubKeys[i] = TestUtils.deriveValidatorPubKeyBytes(keys[i]);
            vm.prank(validators[i]);
            saDiamond.manager().join{value: 10}(pubKeys[i], 10);
        }

        vm.deal(address(saDiamond), 100 ether);
        vm.prank(address(saDiamond));
        gatewayDiamond.manager().register{value: DEFAULT_MIN_VALIDATOR_STAKE + 3 * DEFAULT_CROSS_MSG_FEE}(
            3 * DEFAULT_CROSS_MSG_FEE,
            DEFAULT_MIN_VALIDATOR_STAKE
        );

        SubnetID memory localSubnetID = saDiamond.getter().getParent().createSubnetId(address(saDiamond));

        IpcEnvelope[] memory msgs = new IpcEnvelope[](MAX_MSGS_PER_BATCH);
        for (uint256 i = 0; i < MAX_MSGS_PER_BATCH; i++) {
            IpcEnvelope memory crossMsg = TestUtils.newXnetCallMsg(
                IPCAddress({subnetId: localSubnetID, rawAddress: FvmAddressHelper.from(address(saDiamond))}),
                IPCAddress({
                    subnetId: saDiamond.getter().getParent(),
                    rawAddress: FvmAddressHelper.from(address(saDiamond))
                }),
                1,
                uint64(i)
            );
            msgs[i] = crossMsg;
        }

        vm.prank(validators[0]);

        // submit a full msg batch, even though next expected height is bottomUpCheckPeriod()
        BottomUpCheckpoint memory checkpoint = BottomUpCheckpoint({
            subnetID: localSubnetID,
            blockHeight: 1,
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 0,
            msgs: msgs,
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });
        submitCheckpointInternal(checkpoint, validators, signatures, keys);
        require(saDiamond.getter().lastBottomUpCheckpointHeight() == 1, " checkpoint height incorrect");

        // submit a full msg batch, allow early submission,
        // even though next expected height is bottomUpCheckPeriod()
        checkpoint = BottomUpCheckpoint({
            subnetID: localSubnetID,
            blockHeight: 3,
            blockHash: keccak256("block2"),
            nextConfigurationNumber: 0,
            msgs: msgs,
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });
        submitCheckpointInternal(checkpoint, validators, signatures, keys);
        require(saDiamond.getter().lastBottomUpCheckpointHeight() == 3, " checkpoint height incorrect");

        // should not allow submission of past checkpoints already confirmed, last bottom up checkpoint height is 3
        checkpoint = BottomUpCheckpoint({
            subnetID: localSubnetID,
            blockHeight: 2,
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 0,
            msgs: msgs,
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });
        vm.expectRevert(BottomUpCheckpointAlreadySubmitted.selector);
        submitCheckpointInternal(checkpoint, validators, signatures, keys);

        // submit future checkpoint, should reject
        checkpoint = BottomUpCheckpoint({
            subnetID: localSubnetID,
            blockHeight: saDiamond.getter().bottomUpCheckPeriod() + 1,
            blockHash: keccak256("block2"),
            nextConfigurationNumber: 0,
            msgs: msgs,
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });
        vm.expectRevert(CannotSubmitFutureCheckpoint.selector);
        submitCheckpointInternal(checkpoint, validators, signatures, keys);

        checkpoint = BottomUpCheckpoint({
            subnetID: localSubnetID,
            blockHeight: saDiamond.getter().bottomUpCheckPeriod(),
            blockHash: keccak256("block2"),
            nextConfigurationNumber: 0,
            msgs: new IpcEnvelope[](0),
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });
        submitCheckpointInternal(checkpoint, validators, signatures, keys);
        require(
            saDiamond.getter().lastBottomUpCheckpointHeight() == saDiamond.getter().bottomUpCheckPeriod(),
            " checkpoint height incorrect"
        );

        checkpoint = BottomUpCheckpoint({
            subnetID: localSubnetID,
            blockHeight: saDiamond.getter().bottomUpCheckPeriod() + 1,
            blockHash: keccak256("block2"),
            nextConfigurationNumber: 0,
            msgs: msgs,
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });
        submitCheckpointInternal(checkpoint, validators, signatures, keys);
        require(
            saDiamond.getter().lastBottomUpCheckpointHeight() == saDiamond.getter().bottomUpCheckPeriod() + 1,
            " checkpoint height incorrect"
        );

        checkpoint = BottomUpCheckpoint({
            subnetID: localSubnetID,
            blockHeight: saDiamond.getter().bottomUpCheckPeriod() + 2,
            blockHash: keccak256("block2"),
            nextConfigurationNumber: 0,
            msgs: msgs,
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });
        submitCheckpointInternal(checkpoint, validators, signatures, keys);
        require(
            saDiamond.getter().lastBottomUpCheckpointHeight() == saDiamond.getter().bottomUpCheckPeriod() + 2,
            " checkpoint height incorrect"
        );

        checkpoint = BottomUpCheckpoint({
            subnetID: localSubnetID,
            blockHeight: saDiamond.getter().bottomUpCheckPeriod() + 3,
            blockHash: keccak256("block2"),
            nextConfigurationNumber: 0,
            msgs: new IpcEnvelope[](0),
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });
        vm.expectRevert(InvalidCheckpointEpoch.selector);
        submitCheckpointInternal(checkpoint, validators, signatures, keys);

        checkpoint = BottomUpCheckpoint({
            subnetID: localSubnetID,
            blockHeight: saDiamond.getter().bottomUpCheckPeriod() * 2,
            blockHash: keccak256("block2"),
            nextConfigurationNumber: 0,
            msgs: new IpcEnvelope[](0),
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });
        submitCheckpointInternal(checkpoint, validators, signatures, keys);
        require(
            saDiamond.getter().lastBottomUpCheckpointHeight() == saDiamond.getter().bottomUpCheckPeriod() * 2,
            " checkpoint height incorrect"
        );

        checkpoint = BottomUpCheckpoint({
            subnetID: localSubnetID,
            blockHeight: saDiamond.getter().bottomUpCheckPeriod() * 3,
            blockHash: keccak256("block2"),
            nextConfigurationNumber: 0,
            msgs: new IpcEnvelope[](0),
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });
        submitCheckpointInternal(checkpoint, validators, signatures, keys);
        require(
            saDiamond.getter().lastBottomUpCheckpointHeight() == saDiamond.getter().bottomUpCheckPeriod() * 3,
            " checkpoint height incorrect"
        );
    }

    function testSubnetActorDiamond_submitCheckpointWithReward() public {
        (uint256[] memory keys, address[] memory validators, ) = TestUtils.getThreeValidators(vm);
        bytes[] memory pubKeys = new bytes[](3);
        bytes[] memory signatures = new bytes[](3);

        for (uint256 i = 0; i < 3; i++) {
            vm.deal(validators[i], 10 gwei);
            pubKeys[i] = TestUtils.deriveValidatorPubKeyBytes(keys[i]);
            vm.prank(validators[i]);
            saDiamond.manager().join{value: 10}(pubKeys[i], 10);
        }

        SubnetID memory localSubnetID = saDiamond.getter().getParent().createSubnetId(address(saDiamond));

        // send the first checkpoint
        IpcEnvelope memory crossMsg = TestUtils.newXnetCallMsg(
            IPCAddress({subnetId: localSubnetID, rawAddress: FvmAddressHelper.from(address(saDiamond))}),
            IPCAddress({
                subnetId: saDiamond.getter().getParent(),
                rawAddress: FvmAddressHelper.from(address(saDiamond))
            }),
            DEFAULT_CROSS_MSG_FEE + 1,
            0
        );
        IpcEnvelope[] memory msgs = new IpcEnvelope[](1);
        msgs[0] = crossMsg;

        BottomUpCheckpoint memory checkpoint = BottomUpCheckpoint({
            subnetID: localSubnetID,
            blockHeight: saDiamond.getter().bottomUpCheckPeriod(),
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 0,
            msgs: msgs,
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });

        vm.deal(address(saDiamond), 100 ether);
        vm.prank(address(saDiamond));
        gatewayDiamond.manager().register{value: DEFAULT_MIN_VALIDATOR_STAKE + 6 * DEFAULT_CROSS_MSG_FEE}(
            6 * DEFAULT_CROSS_MSG_FEE,
            DEFAULT_MIN_VALIDATOR_STAKE
        );

        bytes32 hash = keccak256(abi.encode(checkpoint));

        for (uint256 i = 0; i < 3; i++) {
            (uint8 v, bytes32 r, bytes32 s) = vm.sign(keys[i], hash);
            signatures[i] = abi.encodePacked(r, s, v);
        }

        vm.expectCall(gatewayAddress, abi.encodeCall(IGateway.commitCheckpoint, (checkpoint)), 1);
        vm.prank(validators[0]);
        saDiamond.checkpointer().submitCheckpoint(checkpoint, validators, signatures);

        require(
            saDiamond.getter().lastBottomUpCheckpointHeight() == saDiamond.getter().bottomUpCheckPeriod(),
            " checkpoint height correct"
        );

        // send the second checkpoint
        crossMsg = TestUtils.newXnetCallMsg(
            IPCAddress({subnetId: localSubnetID, rawAddress: FvmAddressHelper.from(address(saDiamond))}),
            IPCAddress({
                subnetId: saDiamond.getter().getParent(),
                rawAddress: FvmAddressHelper.from(address(saDiamond))
            }),
            DEFAULT_CROSS_MSG_FEE + 1,
            1
        );
        msgs[0] = crossMsg;

        checkpoint = BottomUpCheckpoint({
            subnetID: localSubnetID,
            blockHeight: 2 * saDiamond.getter().bottomUpCheckPeriod(),
            blockHash: keccak256("block2"),
            nextConfigurationNumber: 0,
            msgs: msgs,
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });

        hash = keccak256(abi.encode(checkpoint));

        for (uint256 i = 0; i < 3; i++) {
            (uint8 v, bytes32 r, bytes32 s) = vm.sign(keys[i], hash);
            signatures[i] = abi.encodePacked(r, s, v);
        }

        vm.prank(validators[0]);
        saDiamond.checkpointer().submitCheckpoint(checkpoint, validators, signatures);
    }

    function testSubnetActorDiamond_DiamondCut() public {
        // add method getNum to subnet actor diamond and assert it can be correctly called
        // replace method getNum and assert it was correctly updated
        // delete method getNum and assert it no longer is callable
        // assert that diamondCut cannot be called by non-owner

        NumberContractFacetSeven ncFacetA = new NumberContractFacetSeven();
        NumberContractFacetEight ncFacetB = new NumberContractFacetEight();

        DiamondCutFacet saDiamondCutter = DiamondCutFacet(address(saDiamond));
        IDiamond.FacetCut[] memory saDiamondCut = new IDiamond.FacetCut[](1);

        bytes4[] memory ncGetterSelectors = new bytes4[](1);
        ncGetterSelectors[0] = NumberContractFacetSeven.getNum.selector;

        saDiamondCut[0] = (
            IDiamond.FacetCut({
                facetAddress: address(ncFacetA),
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: ncGetterSelectors
            })
        );
        //test that other user cannot call diamondcut to add function
        vm.prank(0x1234567890123456789012345678901234567890);
        vm.expectRevert(NotOwner.selector);
        saDiamondCutter.diamondCut(saDiamondCut, address(0), new bytes(0));

        saDiamondCutter.diamondCut(saDiamondCut, address(0), new bytes(0));

        NumberContractFacetSeven saNumberContract = NumberContractFacetSeven(address(saDiamond));
        assert(saNumberContract.getNum() == 7);

        ncGetterSelectors = new bytes4[](1);
        ncGetterSelectors[0] = NumberContractFacetEight.getNum.selector;
        saDiamondCut[0] = (
            IDiamond.FacetCut({
                facetAddress: address(ncFacetB),
                action: IDiamond.FacetCutAction.Replace,
                functionSelectors: ncGetterSelectors
            })
        );

        //test that other user cannot call diamondcut to replace function
        vm.prank(0x1234567890123456789012345678901234567890);
        vm.expectRevert(NotOwner.selector);
        saDiamondCutter.diamondCut(saDiamondCut, address(0), new bytes(0));

        saDiamondCutter.diamondCut(saDiamondCut, address(0), new bytes(0));

        assert(saNumberContract.getNum() == 8);

        //remove facet for getNum
        saDiamondCut[0] = (
            IDiamond.FacetCut({
                facetAddress: 0x0000000000000000000000000000000000000000,
                action: IDiamond.FacetCutAction.Remove,
                functionSelectors: ncGetterSelectors
            })
        );

        //test that other user cannot call diamondcut to remove function
        vm.prank(0x1234567890123456789012345678901234567890);
        vm.expectRevert(NotOwner.selector);
        saDiamondCutter.diamondCut(saDiamondCut, address(0), new bytes(0));

        saDiamondCutter.diamondCut(saDiamondCut, address(0), new bytes(0));

        //assert that calling getNum fails
        vm.expectRevert(abi.encodePacked(FunctionNotFound.selector, ncGetterSelectors));
        saNumberContract.getNum();
    }

    function testSubnetActorDiamond_Unstake() public {
        (address validator, bytes memory publicKey) = TestUtils.deriveValidatorAddress(100);

        vm.expectRevert(CannotReleaseZero.selector);
        vm.prank(validator);
        saDiamond.manager().unstake(0);

        vm.expectRevert(abi.encodeWithSelector(NotValidator.selector, validator));
        vm.prank(validator);
        saDiamond.manager().unstake(10);

        vm.deal(validator, DEFAULT_MIN_VALIDATOR_STAKE);
        vm.prank(validator);
        saDiamond.manager().join{value: DEFAULT_MIN_VALIDATOR_STAKE}(publicKey, DEFAULT_MIN_VALIDATOR_STAKE);
        require(
            saDiamond.getter().getValidator(validator).totalCollateral == DEFAULT_MIN_VALIDATOR_STAKE,
            "initial collateral correct"
        );

        vm.expectRevert(NotEnoughCollateral.selector);
        vm.prank(validator);
        saDiamond.manager().unstake(DEFAULT_MIN_VALIDATOR_STAKE + 100);

        vm.expectRevert(NotEnoughCollateral.selector);
        vm.prank(validator);
        saDiamond.manager().unstake(DEFAULT_MIN_VALIDATOR_STAKE);

        vm.prank(validator);
        saDiamond.manager().unstake(5);
        require(
            saDiamond.getter().getValidator(validator).totalCollateral == DEFAULT_MIN_VALIDATOR_STAKE - 5,
            "collateral correct after unstaking"
        );
    }

    function testSubnetActorDiamond_PreFundRelease_works() public {
        (address validator1, bytes memory publicKey1) = TestUtils.deriveValidatorAddress(100);
        address preFunder = address(102);
        address preReleaser = address(103);

        // total collateral in the gateway
        uint256 collateral = 0;
        uint256 fundAmount = 100;

        require(!saDiamond.getter().isActiveValidator(validator1), "active validator1");
        require(!saDiamond.getter().isWaitingValidator(validator1), "waiting validator1");

        // ======== Step. Join ======

        // pre-fund and pre-release from same address
        vm.startPrank(preReleaser);
        vm.deal(preReleaser, 2 * fundAmount);
        saDiamond.manager().preFund{value: 2 * fundAmount}(2 * fundAmount);
        require(saDiamond.getter().genesisCircSupply() == 2 * fundAmount, "genesis circ supply not correct");
        saDiamond.manager().preRelease(fundAmount);
        require(saDiamond.getter().genesisCircSupply() == fundAmount, "genesis circ supply not correct");
        (address[] memory genesisAddrs, ) = saDiamond.getter().genesisBalances();
        require(genesisAddrs.length == 1, "not one genesis addresses");
        // cannot release more than the initial balance of the address
        vm.expectRevert(NotEnoughBalance.selector);
        saDiamond.manager().preRelease(2 * fundAmount);
        // release all
        saDiamond.manager().preRelease(fundAmount);
        (genesisAddrs, ) = saDiamond.getter().genesisBalances();
        require(saDiamond.getter().genesisCircSupply() == 0, "genesis circ supply not correct");
        require(genesisAddrs.length == 0, "not zero genesis addresses");
        vm.stopPrank();

        // pre-fund from validator and from pre-funder
        vm.startPrank(validator1);
        vm.deal(validator1, fundAmount);
        saDiamond.manager().preFund{value: fundAmount}(fundAmount);
        vm.stopPrank();

        vm.startPrank(preFunder);
        vm.deal(preFunder, fundAmount);
        saDiamond.manager().preFund{value: fundAmount}(fundAmount);
        vm.stopPrank();

        // initial validator joins
        vm.deal(validator1, DEFAULT_MIN_VALIDATOR_STAKE);

        vm.startPrank(validator1);
        saDiamond.manager().join{value: DEFAULT_MIN_VALIDATOR_STAKE}(publicKey1, DEFAULT_MIN_VALIDATOR_STAKE);
        vm.stopPrank();
        collateral = DEFAULT_MIN_VALIDATOR_STAKE;

        require(
            gatewayAddress.balance == collateral + 2 * fundAmount,
            "gw balance is incorrect after validator1 joining"
        );

        require(saDiamond.getter().genesisCircSupply() == 2 * fundAmount, "genesis circ supply not correct");
        (genesisAddrs, ) = saDiamond.getter().genesisBalances();
        require(genesisAddrs.length == 2, "not two genesis addresses");

        // collateral confirmed immediately and network boostrapped
        ValidatorInfo memory v = saDiamond.getter().getValidator(validator1);
        require(v.totalCollateral == DEFAULT_MIN_VALIDATOR_STAKE, "total collateral not expected");
        require(v.confirmedCollateral == DEFAULT_MIN_VALIDATOR_STAKE, "confirmed collateral not equal to collateral");
        require(saDiamond.getter().isActiveValidator(validator1), "not active validator 1");
        require(!saDiamond.getter().isWaitingValidator(validator1), "waiting validator 1");
        TestUtils.ensureBytesEqual(v.metadata, publicKey1);
        require(saDiamond.getter().bootstrapped(), "subnet not bootstrapped");
        require(!saDiamond.getter().killed(), "subnet killed");
        require(saDiamond.getter().genesisValidators().length == 1, "not one validator in genesis");

        (uint64 nextConfigNum, uint64 startConfigNum) = saDiamond.getter().getConfigurationNumbers();
        require(nextConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER, "next config num not 1");
        require(startConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER, "start config num not 1");

        // pre-fund not allowed with bootstrapped subnet
        vm.startPrank(preFunder);
        vm.expectRevert(SubnetAlreadyBootstrapped.selector);
        vm.deal(preFunder, fundAmount);
        saDiamond.manager().preFund{value: fundAmount}(fundAmount);
        vm.stopPrank();
    }

    function testSubnetActorDiamond_PreFundAndLeave_works() public {
        (address validator1, bytes memory publicKey1) = TestUtils.deriveValidatorAddress(100);

        // total collateral in the gateway
        uint256 collateral = DEFAULT_MIN_VALIDATOR_STAKE - 100;
        uint256 fundAmount = 100;

        // pre-fund from validator
        vm.startPrank(validator1);
        vm.deal(validator1, fundAmount);
        saDiamond.manager().preFund{value: fundAmount}(fundAmount);
        vm.stopPrank();

        // initial validator joins but doesn't bootstrap the subnet
        vm.deal(validator1, collateral);
        vm.startPrank(validator1);
        saDiamond.manager().join{value: collateral}(publicKey1, collateral);
        require(
            address(saDiamond).balance == collateral + fundAmount,
            "subnet balance is incorrect after validator1 joining"
        );
        require(saDiamond.getter().genesisCircSupply() == fundAmount, "genesis circ supply not correct");
        (address[] memory genesisAddrs, ) = saDiamond.getter().genesisBalances();
        require(genesisAddrs.length == 1, "not one genesis addresses");

        // Leave should return the collateral and the initial balance
        saDiamond.manager().leave();
        require(address(saDiamond).balance == 0, "subnet balance is incorrect after validator1 leaving");
        require(saDiamond.getter().genesisCircSupply() == 0, "genesis circ supply not zero");
        (genesisAddrs, ) = saDiamond.getter().genesisBalances();
        require(genesisAddrs.length == 0, "not zero genesis addresses");
        vm.stopPrank();

        // initial validator joins to bootstrap the subnet
        vm.deal(validator1, DEFAULT_MIN_VALIDATOR_STAKE);

        vm.startPrank(validator1);
        saDiamond.manager().join{value: DEFAULT_MIN_VALIDATOR_STAKE}(publicKey1, DEFAULT_MIN_VALIDATOR_STAKE);
        vm.stopPrank();

        // pre-release not allowed with bootstrapped subnet
        vm.startPrank(validator1);
        vm.expectRevert(SubnetAlreadyBootstrapped.selector);
        saDiamond.manager().preRelease(fundAmount);
        vm.stopPrank();
    }

    function testSubnetActorDiamond_MultipleJoins_Works_GetValidators() public {
        uint256 n = 10;

        (address[] memory validators, uint256[] memory privKeys, bytes[] memory publicKeys) = TestUtils.newValidators(
            n
        );

        for (uint i = 0; i < n; i++) {
            vm.deal(validators[i], 100 * DEFAULT_MIN_VALIDATOR_STAKE);
        }

        vm.prank(validators[0]);
        saDiamond.manager().join{value: 100 * DEFAULT_MIN_VALIDATOR_STAKE}(
            publicKeys[0],
            100 * DEFAULT_MIN_VALIDATOR_STAKE
        );

        for (uint i = 1; i < n; i++) {
            vm.prank(validators[i]);
            saDiamond.manager().join{value: DEFAULT_MIN_VALIDATOR_STAKE}(publicKeys[i], DEFAULT_MIN_VALIDATOR_STAKE);
        }

        confirmChange(validators[0], privKeys[0]);

        for (uint i = 0; i < n; i++) {
            require(saDiamond.getter().isActiveValidator(validators[i]), "not active validator");
        }
    }

    function testSubnetActorDiamond_Join_Works_WithMinimalStake() public {
        uint256 n = 10;

        (address[] memory validators, uint256[] memory privKeys, bytes[] memory publicKeys) = TestUtils.newValidators(
            n
        );

        vm.deal(validators[0], 100 * DEFAULT_MIN_VALIDATOR_STAKE);
        for (uint i = 1; i < n; i++) {
            vm.deal(validators[i], 1);
        }

        vm.prank(validators[0]);
        saDiamond.manager().join{value: 100 * DEFAULT_MIN_VALIDATOR_STAKE}(
            publicKeys[0],
            100 * DEFAULT_MIN_VALIDATOR_STAKE
        );

        for (uint i = 1; i < n; i++) {
            vm.prank(validators[i]);
            saDiamond.manager().join{value: 1}(publicKeys[i], 1);
        }

        confirmChange(validators[0], privKeys[0]);

        for (uint i = 0; i < n; i++) {
            require(saDiamond.getter().isActiveValidator(validators[i]), "not active validator");
        }
    }

    function testSubnetActorDiamond_NotBootstrapped_LessThanActivation() public {
        uint256 n = 10;

        (address[] memory validators, , bytes[] memory publicKeys) = TestUtils.newValidators(n);

        for (uint i = 0; i < n; i++) {
            vm.deal(validators[i], 1);
            vm.prank(validators[i]);
            saDiamond.manager().join{value: 1}(publicKeys[i], 1);
        }

        require(!saDiamond.getter().bootstrapped());
    }

    function test_second_validator_can_join() public {
        (address validatorAddress1, uint256 privKey1, bytes memory publicKey1) = TestUtils.newValidator(101);
        (address validatorAddress2, , bytes memory publicKey2) = TestUtils.newValidator(102);

        join(validatorAddress1, publicKey1);

        require(saDiamond.getter().bootstrapped(), "subnet not bootstrapped");
        require(saDiamond.getter().isActiveValidator(validatorAddress1), "validator 1 is not active");
        require(!saDiamond.getter().isActiveValidator(validatorAddress2), "validator 2 is active");

        join(validatorAddress2, publicKey2);
        confirmChange(validatorAddress1, privKey1);
        require(saDiamond.getter().isActiveValidator(validatorAddress2), "validator 2 is not active");
    }

    function callback() public view {
        // console.log("callback called");
    }

    function testSubnetActorDiamond_StaticValidation_cannotJoin() public {
        gatewayAddress = address(gatewayDiamond);

        createSubnetActor(
            gatewayAddress,
            ConsensusType.Fendermint,
            DEFAULT_MIN_VALIDATOR_STAKE,
            DEFAULT_MIN_VALIDATORS,
            DEFAULT_CHECKPOINT_PERIOD,
            DEFAULT_MAJORITY_PERCENTAGE,
            PermissionMode.Static,
            2
        );

        (address validator1, bytes memory publicKey1) = TestUtils.deriveValidatorAddress(100);
        vm.deal(validator1, DEFAULT_MIN_VALIDATOR_STAKE * 2);
        vm.prank(validator1);
        saDiamond.manager().join{value: DEFAULT_MIN_VALIDATOR_STAKE / 2}(publicKey1, DEFAULT_MIN_VALIDATOR_STAKE / 2);

        (address validator2, bytes memory publicKey2) = TestUtils.deriveValidatorAddress(101);
        vm.deal(validator2, DEFAULT_MIN_VALIDATOR_STAKE * 2);
        vm.prank(validator2);
        saDiamond.manager().join{value: DEFAULT_MIN_VALIDATOR_STAKE / 2}(publicKey2, DEFAULT_MIN_VALIDATOR_STAKE / 2);

        require(saDiamond.getter().isActiveValidator(validator1), "not active validator 1");
        require(saDiamond.getter().isActiveValidator(validator2), "not active validator 2");

        // cannot join after bootstrap

        vm.expectRevert(abi.encodeWithSelector(MethodNotAllowed.selector, ERR_PERMISSIONED_AND_BOOTSTRAPPED));
        vm.prank(validator1);
        saDiamond.manager().join{value: DEFAULT_MIN_VALIDATOR_STAKE}(publicKey1, DEFAULT_MIN_VALIDATOR_STAKE);

        vm.expectRevert(abi.encodeWithSelector(MethodNotAllowed.selector, ERR_PERMISSIONED_AND_BOOTSTRAPPED));
        (address[] memory validators, , bytes[] memory publicKeys) = TestUtils.newValidators(3);
        uint256[] memory powers = new uint256[](3);
        powers[0] = 10000;
        powers[1] = 20000;
        powers[2] = 5000; // we only have 2 active validators, validator 2 does not have enough power
        saDiamond.manager().setFederatedPower(validators, publicKeys, powers);
    }

    function testSubnetActorDiamond_registration_policy() public {
        (address validator1, bytes memory publicKey1) = TestUtils.deriveValidatorAddress(100);
        vm.deal(validator1, DEFAULT_MIN_VALIDATOR_STAKE);

        vm.expectRevert(abi.encodeWithSelector(MethodNotAllowed.selector, ERR_VALIDATOR_NOT_JOINED));
        vm.prank(validator1);
        saDiamond.manager().stake{value: DEFAULT_MIN_VALIDATOR_STAKE / 2}(DEFAULT_MIN_VALIDATOR_STAKE / 2);

        vm.prank(validator1);
        saDiamond.manager().join{value: DEFAULT_MIN_VALIDATOR_STAKE / 2}(publicKey1, DEFAULT_MIN_VALIDATOR_STAKE / 2);

        require(saDiamond.getter().isActiveValidator(validator1), "active validator 1");
        require(!saDiamond.getter().bootstrapped(), "subnet bootstrapped");

        vm.expectRevert(abi.encodeWithSelector(MethodNotAllowed.selector, ERR_VALIDATOR_JOINED));
        vm.prank(validator1);
        saDiamond.manager().join{value: DEFAULT_MIN_VALIDATOR_STAKE / 2}(publicKey1, DEFAULT_MIN_VALIDATOR_STAKE / 2);

        vm.prank(validator1);
        saDiamond.manager().stake{value: DEFAULT_MIN_VALIDATOR_STAKE / 2}(DEFAULT_MIN_VALIDATOR_STAKE / 2);

        require(saDiamond.getter().isActiveValidator(validator1), "active validator 1");

        (address validator2, bytes memory publicKey2) = TestUtils.deriveValidatorAddress(101);
        vm.deal(validator2, DEFAULT_MIN_VALIDATOR_STAKE * 2);
        vm.prank(validator2);
        saDiamond.manager().join{value: DEFAULT_MIN_VALIDATOR_STAKE / 2}(publicKey2, DEFAULT_MIN_VALIDATOR_STAKE / 2);

        require(saDiamond.getter().isActiveValidator(validator1), "not active validator 1");
        require(saDiamond.getter().bootstrapped(), "subnet not bootstrapped");
    }

    function testSubnetActorDiamond_FederatedValidation_bootstrapWorks() public {
        gatewayAddress = address(gatewayDiamond);

        createSubnetActor(
            gatewayAddress,
            ConsensusType.Fendermint,
            DEFAULT_MIN_VALIDATOR_STAKE,
            DEFAULT_MIN_VALIDATORS,
            DEFAULT_CHECKPOINT_PERIOD,
            DEFAULT_MAJORITY_PERCENTAGE,
            PermissionMode.Federated,
            2
        );

        (address[] memory validators, , bytes[] memory publicKeys) = TestUtils.newValidators(2);

        uint256[] memory powers = new uint256[](2);
        powers[0] = 10000;
        powers[1] = 20000;

        saDiamond.manager().setFederatedPower(validators, publicKeys, powers);
        require(saDiamond.getter().isActiveValidator(validators[0]), "not active validator 1");
        require(saDiamond.getter().isActiveValidator(validators[1]), "not active validator 2");
    }

    function testSubnetActorDiamond_FederatedValidation_bootstrapNotOwnerOfPublicKeys() public {
        gatewayAddress = address(gatewayDiamond);

        createSubnetActor(
            gatewayAddress,
            ConsensusType.Fendermint,
            DEFAULT_MIN_VALIDATOR_STAKE,
            DEFAULT_MIN_VALIDATORS,
            DEFAULT_CHECKPOINT_PERIOD,
            DEFAULT_MAJORITY_PERCENTAGE,
            PermissionMode.Federated,
            2
        );

        (address[] memory validators, , bytes[] memory publicKeys) = TestUtils.newValidators(2);
        publicKeys[1] = publicKeys[0];

        uint256[] memory powers = new uint256[](2);
        powers[0] = 10000;
        powers[1] = 20000;

        vm.expectRevert(abi.encodeWithSelector(NotOwnerOfPublicKey.selector));
        saDiamond.manager().setFederatedPower(validators, publicKeys, powers);
    }

    function testSubnetActorDiamond_FederatedValidation_bootstrapNotEnoughValidators() public {
        gatewayAddress = address(gatewayDiamond);

        createSubnetActor(
            gatewayAddress,
            ConsensusType.Fendermint,
            DEFAULT_MIN_VALIDATOR_STAKE,
            2,
            DEFAULT_CHECKPOINT_PERIOD,
            DEFAULT_MAJORITY_PERCENTAGE,
            PermissionMode.Federated,
            2
        );

        (address[] memory validators, , bytes[] memory publicKeys) = TestUtils.newValidators(1);

        uint256[] memory powers = new uint256[](1);
        powers[0] = 10000;

        vm.expectRevert(abi.encodeWithSelector(NotEnoughGenesisValidators.selector));
        saDiamond.manager().setFederatedPower(validators, publicKeys, powers);
    }

    function testSubnetActorDiamond_FederatedValidation_bootstrapDuplicates() public {
        gatewayAddress = address(gatewayDiamond);

        createSubnetActor(
            gatewayAddress,
            ConsensusType.Fendermint,
            DEFAULT_MIN_VALIDATOR_STAKE,
            DEFAULT_MIN_VALIDATORS,
            DEFAULT_CHECKPOINT_PERIOD,
            DEFAULT_MAJORITY_PERCENTAGE,
            PermissionMode.Federated,
            2
        );

        (address[] memory validators, , bytes[] memory publicKeys) = TestUtils.newValidators(2);
        validators[1] = validators[0];
        publicKeys[1] = publicKeys[0];

        uint256[] memory powers = new uint256[](2);
        powers[0] = 10000;
        powers[1] = 20000;

        vm.expectRevert(abi.encodeWithSelector(DuplicatedGenesisValidator.selector));
        saDiamond.manager().setFederatedPower(validators, publicKeys, powers);
    }

    function testSubnetActorDiamond_FederatedValidation_cannotJoin() public {
        gatewayAddress = address(gatewayDiamond);

        createSubnetActor(
            gatewayAddress,
            ConsensusType.Fendermint,
            DEFAULT_MIN_VALIDATOR_STAKE,
            DEFAULT_MIN_VALIDATORS,
            DEFAULT_CHECKPOINT_PERIOD,
            DEFAULT_MAJORITY_PERCENTAGE,
            PermissionMode.Federated,
            2
        );

        (address[] memory validators, , bytes[] memory publicKeys) = TestUtils.newValidators(2);
        uint256[] memory powers = new uint256[](2);
        powers[0] = 10000;
        powers[1] = 20000;

        saDiamond.manager().setFederatedPower(validators, publicKeys, powers);

        vm.deal(validators[0], DEFAULT_MIN_VALIDATOR_STAKE * 2);
        vm.startPrank(validators[0]);
        vm.expectRevert(abi.encodeWithSelector(MethodNotAllowed.selector, ERR_PERMISSIONED_AND_BOOTSTRAPPED));
        saDiamond.manager().join{value: DEFAULT_MIN_VALIDATOR_STAKE}(publicKeys[0], DEFAULT_MIN_VALIDATOR_STAKE);
    }

    function testSubnetActorDiamond_FederatedValidation_works() public {
        gatewayAddress = address(gatewayDiamond);

        createSubnetActor(
            gatewayAddress,
            ConsensusType.Fendermint,
            DEFAULT_MIN_VALIDATOR_STAKE,
            DEFAULT_MIN_VALIDATORS,
            DEFAULT_CHECKPOINT_PERIOD,
            DEFAULT_MAJORITY_PERCENTAGE,
            PermissionMode.Federated,
            2
        );

        (address[] memory validators, uint256[] memory privKeys, bytes[] memory publicKeys) = TestUtils.newValidators(
            3
        );
        uint256[] memory powers = new uint256[](3);
        powers[0] = 10000;
        powers[1] = 20000;
        powers[2] = 5000; // we only have 2 active validators, validator 2 does not have enough power

        saDiamond.manager().setFederatedPower(validators, publicKeys, powers);

        require(saDiamond.getter().isActiveValidator(validators[0]), "not active validator 0");
        require(saDiamond.getter().isActiveValidator(validators[1]), "not active validator 1");
        require(!saDiamond.getter().isActiveValidator(validators[2]), "2 should not be active validator");

        // change in validator power
        powers[2] = 10001;

        saDiamond.manager().setFederatedPower(validators, publicKeys, powers);

        confirmChange(validators[0], privKeys[0], validators[1], privKeys[1]);

        require(!saDiamond.getter().isActiveValidator(validators[0]), "0 should not be active validator");
        require(saDiamond.getter().isActiveValidator(validators[1]), "not active validator 1");
        require(saDiamond.getter().isActiveValidator(validators[2]), "not active validator 2");

        /// reduce validator 2 power
        powers[2] = 5000;

        saDiamond.manager().setFederatedPower(validators, publicKeys, powers);

        confirmChange(validators[2], privKeys[2], validators[1], privKeys[1]);

        require(saDiamond.getter().isActiveValidator(validators[0]), "not active validator 0");
        require(saDiamond.getter().isActiveValidator(validators[1]), "not active validator 1");
        require(!saDiamond.getter().isActiveValidator(validators[2]), "2 should not be active validator");
    }

    function testSubnetActorDiamond_FederatedValidation_worksWithDuplicates() public {
        gatewayAddress = address(gatewayDiamond);

        createSubnetActor(
            gatewayAddress,
            ConsensusType.Fendermint,
            DEFAULT_MIN_VALIDATOR_STAKE,
            DEFAULT_MIN_VALIDATORS,
            DEFAULT_CHECKPOINT_PERIOD,
            DEFAULT_MAJORITY_PERCENTAGE,
            PermissionMode.Federated,
            2
        );

        (address[] memory validators, uint256[] memory privKeys, bytes[] memory publicKeys) = TestUtils.newValidators(
            3
        );
        uint256[] memory powers = new uint256[](3);
        powers[0] = 10000;
        powers[1] = 20000;
        powers[2] = 5000; // we only have 2 active validators, validator 2 does not have enough power

        saDiamond.manager().setFederatedPower(validators, publicKeys, powers);

        require(saDiamond.getter().isActiveValidator(validators[0]), "not active validator 0");
        require(saDiamond.getter().isActiveValidator(validators[1]), "not active validator 1");
        require(!saDiamond.getter().isActiveValidator(validators[2]), "2 should not be active validator");

        // change in validator power, changing validator 2's power to 10001.

        // store validator 0's data in new variables
        address prevV = validators[0];
        uint256 prevPrivateKey = privKeys[0];

        // creating duplicates of validator 2's data
        validators[0] = validators[2];
        publicKeys[0] = publicKeys[2];

        // the latest validator 2's power, so we have a duplicate of validator 2's power
        powers[0] = 9999; // lower than validator 0's power, will not kick validator 1 off
        powers[2] = 10001; // higher than validator 0's power, will kick validator 1 off

        saDiamond.manager().setFederatedPower(validators, publicKeys, powers);

        confirmChange(prevV, prevPrivateKey, validators[1], privKeys[1]);

        // we should see validator 0 kicked off
        require(!saDiamond.getter().isActiveValidator(prevV), "0 should not be active validator");
        require(saDiamond.getter().isActiveValidator(validators[1]), "not active validator 1");
        require(saDiamond.getter().isActiveValidator(validators[2]), "not active validator 2");
    }

    // -----------------------------------------------------------------------------------------------------------------
    // Tests for pausable
    // -----------------------------------------------------------------------------------------------------------------

    function testSubnetActorDiamond_Pausable_PauseUnpause() public {
        require(!saDiamond.pauser().paused(), "paused");

        saDiamond.pauser().pause();
        require(saDiamond.pauser().paused(), "not paused");

        saDiamond.pauser().unpause();
        require(!saDiamond.pauser().paused(), "paused");
    }

    function testSubnetActorDiamond_Pausable_EnforcedPause() public {
        saDiamond.pauser().pause();
        require(saDiamond.pauser().paused(), "not paused");

        uint256 n = 1;
        (address[] memory validators, , bytes[] memory publicKeys) = TestUtils.newValidators(n);
        vm.deal(validators[0], 20);

        vm.prank(validators[0]);
        vm.expectRevert(Pausable.EnforcedPause.selector);
        saDiamond.manager().join{value: 10}(publicKeys[0], 10);

        vm.prank(validators[0]);
        vm.expectRevert(Pausable.EnforcedPause.selector);
        saDiamond.manager().stake{value: 10}(10);

        vm.prank(validators[0]);
        vm.expectRevert(Pausable.EnforcedPause.selector);
        saDiamond.manager().unstake(1);

        vm.prank(validators[0]);
        vm.expectRevert(Pausable.EnforcedPause.selector);
        saDiamond.manager().leave();

        vm.prank(validators[0]);
        vm.expectRevert(Pausable.EnforcedPause.selector);
        saDiamond.manager().addBootstrapNode("1.1.1.1");

        // Test on submitCheckpoint() reverts if the contract is paused
        // is in testSubnetActorDiamond_submitCheckpoint_basic.
    }

    function testSubnetActorDiamond_PauseUnpause_NotOwner() public {
        vm.prank(vm.addr(1));
        vm.expectRevert(NotOwner.selector);
        saDiamond.pauser().pause();

        saDiamond.pauser().pause();
        require(saDiamond.pauser().paused(), "not paused");

        vm.prank(vm.addr(1));
        vm.expectRevert(NotOwner.selector);
        saDiamond.pauser().unpause();

        saDiamond.pauser().unpause();
        require(!saDiamond.pauser().paused(), "not paused");
    }

    function testSubnetActorDiamond_Pausable_CannotPauseAgain() public {
        saDiamond.pauser().pause();
        require(saDiamond.pauser().paused(), "not paused");

        vm.expectRevert(Pausable.EnforcedPause.selector);
        saDiamond.pauser().pause();
    }

    function testSubnetActorDiamond_Pausable_CannotUnpauseAgain() public {
        vm.expectRevert(Pausable.ExpectedPause.selector);
        saDiamond.pauser().unpause();
        require(!saDiamond.pauser().paused(), "paused");
    }

    // ----------------------------
    // Tests for collateral token
    // ----------------------------
    function testSubnetActorDiamond_CollateralERC20_SupplyERC20_RegisteredInGateway() public {
        (address validator, uint256 privKey, bytes memory publicKey) = TestUtils.newValidator(100);
        (address validator2, , ) = TestUtils.newValidator(101);

        // a bit of gas for execution, should not be needed
        vm.deal(validator, DEFAULT_MIN_VALIDATOR_STAKE - 100);
        vm.deal(validator2, DEFAULT_MIN_VALIDATOR_STAKE - 100);

        ERC20PresetFixedSupply sourceToken = new ERC20PresetFixedSupply("t", "t", 100000000000, validator2);
        ERC20PresetFixedSupply collateralToken = new ERC20PresetFixedSupply(
            "t",
            "t",
            DEFAULT_MIN_VALIDATOR_STAKE * 10,
            validator
        );

        Asset memory source = Asset({kind: AssetKind.ERC20, tokenAddress: address(sourceToken)});
        Asset memory collateral = Asset({kind: AssetKind.ERC20, tokenAddress: address(collateralToken)});

        gatewayAddress = address(gatewayDiamond);
        SubnetActorDiamond.ConstructorParams memory params = defaultSubnetActorParamsWith(
            gatewayAddress,
            SubnetID(ROOTNET_CHAINID, new address[](0)),
            source,
            collateral
        );

        saDiamond = createSubnetActor(params);

        vm.prank(validator2);
        sourceToken.approve(address(saDiamond.manager()), 100);
        vm.prank(validator2);
        saDiamond.manager().preFund(100);

        vm.prank(validator);
        collateralToken.approve(address(saDiamond.manager()), DEFAULT_MIN_VALIDATOR_STAKE * 2);

        vm.prank(validator);
        saDiamond.manager().join(publicKey, DEFAULT_MIN_VALIDATOR_STAKE);
        require(collateralToken.balanceOf(validator) == DEFAULT_MIN_VALIDATOR_STAKE * 9);
        require(collateralToken.balanceOf(address(saDiamond)) == 0);
        require(collateralToken.balanceOf(gatewayAddress) == DEFAULT_MIN_VALIDATOR_STAKE);
        require(sourceToken.balanceOf(gatewayAddress) == 100);

        vm.prank(validator);
        saDiamond.manager().stake(DEFAULT_MIN_VALIDATOR_STAKE);
        require(
            collateralToken.balanceOf(validator) == DEFAULT_MIN_VALIDATOR_STAKE * 8,
            "validator post stake balance wrong"
        );
        require(
            collateralToken.balanceOf(address(saDiamond)) == DEFAULT_MIN_VALIDATOR_STAKE,
            "saDiamond post stake balance wrong"
        );
        require(
            collateralToken.balanceOf(gatewayAddress) == DEFAULT_MIN_VALIDATOR_STAKE,
            "gateway post stake balance wrong"
        );
        confirmChange(validator, privKey);
        require(
            collateralToken.balanceOf(validator) == DEFAULT_MIN_VALIDATOR_STAKE * 8,
            "validator post stake confirmed balance wrong"
        );
        require(collateralToken.balanceOf(address(saDiamond)) == 0, "saDiamond post stake confirmed balance wrong");
        require(
            collateralToken.balanceOf(gatewayAddress) == DEFAULT_MIN_VALIDATOR_STAKE * 2,
            "gateway post stake confirmed balance wrong"
        );

        vm.prank(validator);
        saDiamond.manager().unstake(DEFAULT_MIN_VALIDATOR_STAKE);
        require(
            collateralToken.balanceOf(validator) == DEFAULT_MIN_VALIDATOR_STAKE * 8,
            "validator post unstake balance wrong"
        );
        require(collateralToken.balanceOf(address(saDiamond)) == 0, "saDiamond post unstake balance wrong");
        require(
            collateralToken.balanceOf(gatewayAddress) == DEFAULT_MIN_VALIDATOR_STAKE * 2,
            "gateway post unstake balance wrong"
        );
        confirmChange(validator, privKey);
        require(
            collateralToken.balanceOf(validator) == DEFAULT_MIN_VALIDATOR_STAKE * 8,
            "validator post unstake balance wrong"
        );
        require(
            collateralToken.balanceOf(address(saDiamond)) == DEFAULT_MIN_VALIDATOR_STAKE,
            "saDiamond post unstake balance wrong"
        );
        require(
            collateralToken.balanceOf(gatewayAddress) == DEFAULT_MIN_VALIDATOR_STAKE,
            "gateway post unstake balance wrong"
        );

        vm.prank(validator);
        saDiamond.rewarder().claim();
        require(
            collateralToken.balanceOf(validator) == DEFAULT_MIN_VALIDATOR_STAKE * 9,
            "validator post claim balance wrong"
        );
        require(collateralToken.balanceOf(address(saDiamond)) == 0, "saDiamond post claim balance wrong");
        require(
            collateralToken.balanceOf(gatewayAddress) == DEFAULT_MIN_VALIDATOR_STAKE,
            "gateway post claim balance wrong"
        );

        vm.prank(validator);
        saDiamond.manager().leave();
        require(
            collateralToken.balanceOf(validator) == DEFAULT_MIN_VALIDATOR_STAKE * 9,
            "validator post leave balance wrong"
        );
        require(collateralToken.balanceOf(address(saDiamond)) == 0, "saDiamond post leave balance wrong");
        require(
            collateralToken.balanceOf(gatewayAddress) == DEFAULT_MIN_VALIDATOR_STAKE,
            "gateway post leave balance wrong"
        );
        confirmChange(validator, privKey);
        require(
            collateralToken.balanceOf(validator) == DEFAULT_MIN_VALIDATOR_STAKE * 9,
            "validator confirmed leave balance wrong"
        );
        require(
            collateralToken.balanceOf(address(saDiamond)) == DEFAULT_MIN_VALIDATOR_STAKE,
            "saDiamond confirmed leave balance wrong"
        );
        require(collateralToken.balanceOf(gatewayAddress) == 0, "gateway confirmed leave balance wrong");

        vm.prank(validator);
        saDiamond.rewarder().claim();
        require(
            collateralToken.balanceOf(validator) == DEFAULT_MIN_VALIDATOR_STAKE * 10,
            "validator post leave claim balance wrong"
        );
        require(collateralToken.balanceOf(address(saDiamond)) == 0, "saDiamond post claim balance wrong");
        require(collateralToken.balanceOf(gatewayAddress) == 0, "gateway post leave claim balance wrong");
    }

    function testSubnetActorDiamond_CollateralERC20_SupplyERC20_SameToken_RegisteredInGateway() public {
        (address validator, uint256 privKey, bytes memory publicKey) = TestUtils.newValidator(100);
        (address validator2, , ) = TestUtils.newValidator(101);

        // a bit of gas for execution, should not be needed
        vm.deal(validator, DEFAULT_MIN_VALIDATOR_STAKE - 100);
        vm.deal(validator2, DEFAULT_MIN_VALIDATOR_STAKE - 100);

        ERC20PresetFixedSupply token = new ERC20PresetFixedSupply(
            "t",
            "t",
            DEFAULT_MIN_VALIDATOR_STAKE * 20,
            validator
        );

        Asset memory gt = Asset({kind: AssetKind.ERC20, tokenAddress: address(token)});

        gatewayAddress = address(gatewayDiamond);
        SubnetActorDiamond.ConstructorParams memory params = defaultSubnetActorParamsWith(
            gatewayAddress,
            SubnetID(ROOTNET_CHAINID, new address[](0)),
            gt,
            gt
        );

        saDiamond = createSubnetActor(params);

        vm.prank(validator);
        token.transfer(validator2, DEFAULT_MIN_VALIDATOR_STAKE * 10);

        vm.prank(validator2);
        token.approve(address(saDiamond.manager()), 100);
        vm.prank(validator2);
        saDiamond.manager().preFund(100);

        vm.prank(validator);
        token.approve(address(saDiamond.manager()), DEFAULT_MIN_VALIDATOR_STAKE * 2);

        vm.prank(validator);
        saDiamond.manager().join(publicKey, DEFAULT_MIN_VALIDATOR_STAKE);
        require(token.balanceOf(validator) == DEFAULT_MIN_VALIDATOR_STAKE * 9);
        require(token.balanceOf(address(saDiamond)) == 0);
        require(token.balanceOf(gatewayAddress) == DEFAULT_MIN_VALIDATOR_STAKE + 100);

        vm.prank(validator);
        saDiamond.manager().stake(DEFAULT_MIN_VALIDATOR_STAKE);
        require(token.balanceOf(validator) == DEFAULT_MIN_VALIDATOR_STAKE * 8, "validator post stake balance wrong");
        require(
            token.balanceOf(address(saDiamond)) == DEFAULT_MIN_VALIDATOR_STAKE,
            "saDiamond post stake balance wrong"
        );
        require(
            token.balanceOf(gatewayAddress) == DEFAULT_MIN_VALIDATOR_STAKE + 100,
            "gateway post stake balance wrong"
        );
        confirmChange(validator, privKey);
        require(
            token.balanceOf(validator) == DEFAULT_MIN_VALIDATOR_STAKE * 8,
            "validator post stake confirmed balance wrong"
        );
        require(token.balanceOf(address(saDiamond)) == 0, "saDiamond post stake confirmed balance wrong");
        require(
            token.balanceOf(gatewayAddress) == DEFAULT_MIN_VALIDATOR_STAKE * 2 + 100,
            "gateway post stake confirmed balance wrong"
        );

        vm.prank(validator);
        saDiamond.manager().unstake(DEFAULT_MIN_VALIDATOR_STAKE);
        require(token.balanceOf(validator) == DEFAULT_MIN_VALIDATOR_STAKE * 8, "validator post unstake balance wrong");
        require(token.balanceOf(address(saDiamond)) == 0, "saDiamond post unstake balance wrong");
        require(
            token.balanceOf(gatewayAddress) == DEFAULT_MIN_VALIDATOR_STAKE * 2 + 100,
            "gateway post unstake balance wrong"
        );
        confirmChange(validator, privKey);
        require(token.balanceOf(validator) == DEFAULT_MIN_VALIDATOR_STAKE * 8, "validator post unstake balance wrong");
        require(
            token.balanceOf(address(saDiamond)) == DEFAULT_MIN_VALIDATOR_STAKE,
            "saDiamond post unstake balance wrong"
        );
        require(
            token.balanceOf(gatewayAddress) == DEFAULT_MIN_VALIDATOR_STAKE + 100,
            "gateway post unstake balance wrong"
        );

        vm.prank(validator);
        saDiamond.rewarder().claim();
        require(token.balanceOf(validator) == DEFAULT_MIN_VALIDATOR_STAKE * 9, "validator post claim balance wrong");
        require(token.balanceOf(address(saDiamond)) == 0, "saDiamond post claim balance wrong");
        require(
            token.balanceOf(gatewayAddress) == DEFAULT_MIN_VALIDATOR_STAKE + 100,
            "gateway post claim balance wrong"
        );

        vm.prank(validator);
        saDiamond.manager().leave();
        require(token.balanceOf(validator) == DEFAULT_MIN_VALIDATOR_STAKE * 9, "validator post leave balance wrong");
        require(token.balanceOf(address(saDiamond)) == 0, "saDiamond post leave balance wrong");
        require(
            token.balanceOf(gatewayAddress) == DEFAULT_MIN_VALIDATOR_STAKE + 100,
            "gateway post leave balance wrong"
        );
        confirmChange(validator, privKey);
        require(
            token.balanceOf(validator) == DEFAULT_MIN_VALIDATOR_STAKE * 9,
            "validator confirmed leave balance wrong"
        );
        require(
            token.balanceOf(address(saDiamond)) == DEFAULT_MIN_VALIDATOR_STAKE,
            "saDiamond confirmed leave balance wrong"
        );
        require(token.balanceOf(gatewayAddress) == 100, "gateway confirmed leave balance wrong");

        vm.prank(validator);
        saDiamond.rewarder().claim();
        require(
            token.balanceOf(validator) == DEFAULT_MIN_VALIDATOR_STAKE * 10,
            "validator post leave claim balance wrong"
        );
        require(token.balanceOf(address(saDiamond)) == 0, "saDiamond post claim balance wrong");
        require(token.balanceOf(gatewayAddress) == 100, "gateway post leave claim balance wrong");
    }

    function testSubnetActorDiamond_CollateralERC20_SupplyNative_RegisteredInGateway() public {
        (address validator, uint256 privKey, bytes memory publicKey) = TestUtils.newValidator(100);
        (address validator2, , ) = TestUtils.newValidator(101);

        // a bit of gas for execution, should not be needed
        vm.deal(validator, DEFAULT_MIN_VALIDATOR_STAKE - 100);
        vm.deal(validator2, DEFAULT_MIN_VALIDATOR_STAKE - 100);

        ERC20PresetFixedSupply collateralToken = new ERC20PresetFixedSupply(
            "t",
            "t",
            DEFAULT_MIN_VALIDATOR_STAKE * 10,
            validator
        );

        Asset memory source = AssetHelper.native();
        Asset memory collateral = Asset({kind: AssetKind.ERC20, tokenAddress: address(collateralToken)});

        gatewayAddress = address(gatewayDiamond);
        SubnetActorDiamond.ConstructorParams memory params = defaultSubnetActorParamsWith(
            gatewayAddress,
            SubnetID(ROOTNET_CHAINID, new address[](0)),
            source,
            collateral
        );

        saDiamond = createSubnetActor(params);

        vm.prank(validator2);
        saDiamond.manager().preFund{value: 100}(100);

        vm.prank(validator);
        collateralToken.approve(address(saDiamond.manager()), DEFAULT_MIN_VALIDATOR_STAKE * 2);

        vm.prank(validator);
        saDiamond.manager().join(publicKey, DEFAULT_MIN_VALIDATOR_STAKE);
        require(collateralToken.balanceOf(validator) == DEFAULT_MIN_VALIDATOR_STAKE * 9);
        require(collateralToken.balanceOf(address(saDiamond)) == 0);
        require(collateralToken.balanceOf(gatewayAddress) == DEFAULT_MIN_VALIDATOR_STAKE);
        require(address(gatewayAddress).balance == 100);

        vm.prank(validator);
        saDiamond.manager().stake(DEFAULT_MIN_VALIDATOR_STAKE);
        require(
            collateralToken.balanceOf(validator) == DEFAULT_MIN_VALIDATOR_STAKE * 8,
            "validator post stake balance wrong"
        );
        require(
            collateralToken.balanceOf(address(saDiamond)) == DEFAULT_MIN_VALIDATOR_STAKE,
            "saDiamond post stake balance wrong"
        );
        require(
            collateralToken.balanceOf(gatewayAddress) == DEFAULT_MIN_VALIDATOR_STAKE,
            "gateway post stake balance wrong"
        );
        confirmChange(validator, privKey);
        require(
            collateralToken.balanceOf(validator) == DEFAULT_MIN_VALIDATOR_STAKE * 8,
            "validator post stake confirmed balance wrong"
        );
        require(collateralToken.balanceOf(address(saDiamond)) == 0, "saDiamond post stake confirmed balance wrong");
        require(
            collateralToken.balanceOf(gatewayAddress) == DEFAULT_MIN_VALIDATOR_STAKE * 2,
            "gateway post stake confirmed balance wrong"
        );

        vm.prank(validator);
        saDiamond.manager().unstake(DEFAULT_MIN_VALIDATOR_STAKE);
        require(
            collateralToken.balanceOf(validator) == DEFAULT_MIN_VALIDATOR_STAKE * 8,
            "validator post unstake balance wrong"
        );
        require(collateralToken.balanceOf(address(saDiamond)) == 0, "saDiamond post unstake balance wrong");
        require(
            collateralToken.balanceOf(gatewayAddress) == DEFAULT_MIN_VALIDATOR_STAKE * 2,
            "gateway post unstake balance wrong"
        );
        confirmChange(validator, privKey);
        require(
            collateralToken.balanceOf(validator) == DEFAULT_MIN_VALIDATOR_STAKE * 8,
            "validator post unstake balance wrong"
        );
        require(
            collateralToken.balanceOf(address(saDiamond)) == DEFAULT_MIN_VALIDATOR_STAKE,
            "saDiamond post unstake balance wrong"
        );
        require(
            collateralToken.balanceOf(gatewayAddress) == DEFAULT_MIN_VALIDATOR_STAKE,
            "gateway post unstake balance wrong"
        );

        vm.prank(validator);
        saDiamond.rewarder().claim();
        require(
            collateralToken.balanceOf(validator) == DEFAULT_MIN_VALIDATOR_STAKE * 9,
            "validator post claim balance wrong"
        );
        require(collateralToken.balanceOf(address(saDiamond)) == 0, "saDiamond post claim balance wrong");
        require(
            collateralToken.balanceOf(gatewayAddress) == DEFAULT_MIN_VALIDATOR_STAKE,
            "gateway post claim balance wrong"
        );
    }

    function testSubnetActorDiamond_CollateralNative_SupplyNative_RegisteredInGateway() public {
        (address validator, uint256 privKey, bytes memory publicKey) = TestUtils.newValidator(100);
        (address validator2, , ) = TestUtils.newValidator(101);

        // a bit of gas for execution, should not be needed
        vm.deal(validator, DEFAULT_MIN_VALIDATOR_STAKE * 10);
        vm.deal(validator2, DEFAULT_MIN_VALIDATOR_STAKE - 100);

        gatewayAddress = address(gatewayDiamond);
        SubnetActorDiamond.ConstructorParams memory params = defaultSubnetActorParamsWith(
            gatewayAddress,
            SubnetID(ROOTNET_CHAINID, new address[](0)),
            AssetHelper.native(),
            AssetHelper.native()
        );

        saDiamond = createSubnetActor(params);

        vm.prank(validator2);
        saDiamond.manager().preFund{value: 100}(100);

        vm.prank(validator);
        saDiamond.manager().join{value: DEFAULT_MIN_VALIDATOR_STAKE}(publicKey, DEFAULT_MIN_VALIDATOR_STAKE);
        require(address(validator).balance == DEFAULT_MIN_VALIDATOR_STAKE * 9, "validator post join balance wrong");
        require(address(saDiamond).balance == 0, "saDiamond post join balance wrong");
        require(
            address(gatewayAddress).balance == 100 + DEFAULT_MIN_VALIDATOR_STAKE,
            "gateway post join balance wrong"
        );

        vm.prank(validator);
        saDiamond.manager().stake{value: DEFAULT_MIN_VALIDATOR_STAKE}(DEFAULT_MIN_VALIDATOR_STAKE);
        require(address(validator).balance == DEFAULT_MIN_VALIDATOR_STAKE * 8, "validator post stake balance wrong");
        require(address(saDiamond).balance == DEFAULT_MIN_VALIDATOR_STAKE, "saDiamond post stake balance wrong");
        require(
            address(gatewayAddress).balance == DEFAULT_MIN_VALIDATOR_STAKE + 100,
            "gateway post stake balance wrong"
        );
        confirmChange(validator, privKey);
        require(
            address(validator).balance == DEFAULT_MIN_VALIDATOR_STAKE * 8,
            "validator post stake confirmed balance wrong"
        );
        require(
            address(gatewayAddress).balance == DEFAULT_MIN_VALIDATOR_STAKE * 2 + 100,
            "gateway post stake confirmed balance wrong"
        );

        vm.prank(validator);
        saDiamond.manager().unstake(DEFAULT_MIN_VALIDATOR_STAKE);
        require(address(validator).balance == DEFAULT_MIN_VALIDATOR_STAKE * 8, "validator post unstake balance wrong");
        require(
            address(gatewayAddress).balance == DEFAULT_MIN_VALIDATOR_STAKE * 2 + 100,
            "gateway post unstake balance wrong"
        );
        confirmChange(validator, privKey);
        require(address(validator).balance == DEFAULT_MIN_VALIDATOR_STAKE * 8, "validator post unstake balance wrong");
        require(
            address(gatewayAddress).balance == DEFAULT_MIN_VALIDATOR_STAKE + 100,
            "gateway post unstake balance wrong"
        );

        vm.prank(validator);
        saDiamond.rewarder().claim();
        require(address(validator).balance == DEFAULT_MIN_VALIDATOR_STAKE * 9, "validator post claim balance wrong");
        require(
            address(gatewayAddress).balance == DEFAULT_MIN_VALIDATOR_STAKE + 100,
            "gateway post claim balance wrong"
        );
    }

    function testSubnetActorDiamond_CollateralNative_SupplyERC20_RegisteredInGateway() public {
        (address validator, uint256 privKey, bytes memory publicKey) = TestUtils.newValidator(100);
        (address validator2, , ) = TestUtils.newValidator(101);

        // a bit of gas for execution, should not be needed
        vm.deal(validator, DEFAULT_MIN_VALIDATOR_STAKE * 10);
        vm.deal(validator2, DEFAULT_MIN_VALIDATOR_STAKE - 100);

        ERC20PresetFixedSupply sourceToken = new ERC20PresetFixedSupply(
            "t",
            "t",
            DEFAULT_MIN_VALIDATOR_STAKE * 10,
            validator2
        );
        Asset memory source = Asset({kind: AssetKind.ERC20, tokenAddress: address(sourceToken)});

        gatewayAddress = address(gatewayDiamond);
        SubnetActorDiamond.ConstructorParams memory params = defaultSubnetActorParamsWith(
            gatewayAddress,
            SubnetID(ROOTNET_CHAINID, new address[](0)),
            source,
            AssetHelper.native()
        );

        saDiamond = createSubnetActor(params);

        vm.prank(validator2);
        sourceToken.approve(address(saDiamond.manager()), 100);
        vm.prank(validator2);
        saDiamond.manager().preFund(100);

        vm.prank(validator);
        saDiamond.manager().join{value: DEFAULT_MIN_VALIDATOR_STAKE}(publicKey, DEFAULT_MIN_VALIDATOR_STAKE);
        require(address(validator).balance == DEFAULT_MIN_VALIDATOR_STAKE * 9, "validator post join balance wrong");
        require(address(saDiamond).balance == 0, "saDiamond post join balance wrong");
        require(address(gatewayAddress).balance == DEFAULT_MIN_VALIDATOR_STAKE, "gateway post join balance wrong");
        require(sourceToken.balanceOf(gatewayAddress) == 100, "saDiamond post join balance wrong");

        vm.prank(validator);
        saDiamond.manager().stake{value: DEFAULT_MIN_VALIDATOR_STAKE}(DEFAULT_MIN_VALIDATOR_STAKE);
        require(address(validator).balance == DEFAULT_MIN_VALIDATOR_STAKE * 8, "validator post stake balance wrong");
        require(address(saDiamond).balance == DEFAULT_MIN_VALIDATOR_STAKE, "saDiamond post stake balance wrong");
        require(address(gatewayAddress).balance == DEFAULT_MIN_VALIDATOR_STAKE, "gateway post stake balance wrong");
        confirmChange(validator, privKey);
        require(
            address(validator).balance == DEFAULT_MIN_VALIDATOR_STAKE * 8,
            "validator post stake confirmed balance wrong"
        );
        require(
            address(gatewayAddress).balance == DEFAULT_MIN_VALIDATOR_STAKE * 2,
            "gateway post stake confirmed balance wrong"
        );

        vm.prank(validator);
        saDiamond.manager().unstake(DEFAULT_MIN_VALIDATOR_STAKE);
        require(address(validator).balance == DEFAULT_MIN_VALIDATOR_STAKE * 8, "validator post unstake balance wrong");
        require(
            address(gatewayAddress).balance == DEFAULT_MIN_VALIDATOR_STAKE * 2,
            "gateway post unstake balance wrong"
        );
        confirmChange(validator, privKey);
        require(address(validator).balance == DEFAULT_MIN_VALIDATOR_STAKE * 8, "validator post unstake balance wrong");
        require(address(gatewayAddress).balance == DEFAULT_MIN_VALIDATOR_STAKE, "gateway post unstake balance wrong");

        vm.prank(validator);
        saDiamond.rewarder().claim();
        require(address(validator).balance == DEFAULT_MIN_VALIDATOR_STAKE * 9, "validator post claim balance wrong");
        require(address(gatewayAddress).balance == DEFAULT_MIN_VALIDATOR_STAKE, "gateway post claim balance wrong");
    }

    // ============== Test Activities ===============
    function testSubnetActor_ValidatorClaimMiningReward_Works() public {
        gatewayAddress = address(gatewayDiamond);

        Asset memory source = Asset({kind: AssetKind.Native, tokenAddress: address(0)});

        SubnetActorDiamond.ConstructorParams memory params = defaultSubnetActorParamsWith(
            gatewayAddress,
            SubnetID(ROOTNET_CHAINID, new address[](0)),
            source,
            AssetHelper.native()
        );
        ValidatorRewarderMap m = new ValidatorRewarderMap();
        params.validatorRewarder = address(m);
        params.minValidators = 2;
        params.permissionMode = PermissionMode.Federated;

        saDiamond = createSubnetActor(params);

        SubnetID memory subnetId = SubnetID(ROOTNET_CHAINID, new address[](1));
        subnetId.route[0] = address(saDiamond);
        m.setSubnet(subnetId);

        (address[] memory addrs, uint256[] memory privKeys, bytes[] memory pubkeys) = TestUtils.newValidators(4);

        uint256[] memory powers = new uint256[](4);
        powers[0] = 10000;
        powers[1] = 10000;
        powers[2] = 10000;
        powers[3] = 10000;
        saDiamond.manager().setFederatedPower(addrs, pubkeys, powers);

        uint64[] memory blocksMined = new uint64[](addrs.length);

        blocksMined[0] = 1;
        blocksMined[1] = 2;

        (bytes32 activityRoot, bytes32[][] memory proofs) = MerkleTreeHelper.createMerkleProofsForConsensusActivity(
            addrs,
            blocksMined
        );

        confirmChange(addrs, privKeys, ActivityHelper.newCompressedActivityRollup(2, 3, activityRoot));

        uint64 bottomUpCheckPeriod = uint64(gatewayDiamond.getter().bottomUpCheckPeriod());

        vm.startPrank(addrs[0]);
        vm.deal(addrs[0], 1 ether);
        saDiamond.activity().claim(
            subnetId,
            bottomUpCheckPeriod,
            Consensus.ValidatorData({validator: addrs[0], blocksCommitted: blocksMined[0]}),
            ActivityHelper.wrapBytes32Array(proofs[0])
        );

        vm.startPrank(addrs[1]);
        vm.deal(addrs[1], 1 ether);
        saDiamond.activity().claim(
            subnetId,
            bottomUpCheckPeriod,
            Consensus.ValidatorData({validator: addrs[1], blocksCommitted: blocksMined[1]}),
            ActivityHelper.wrapBytes32Array(proofs[1])
        );

        // These validators have no claims; they were inactive, so the pending activity should've been removed
        // and as a result, the claim should fail.

        vm.startPrank(addrs[2]);
        vm.deal(addrs[2], 1 ether);
        vm.expectRevert(MissingActivityCommitment.selector);
        saDiamond.activity().claim(
            subnetId,
            bottomUpCheckPeriod,
            Consensus.ValidatorData({validator: addrs[2], blocksCommitted: blocksMined[2]}),
            ActivityHelper.wrapBytes32Array(proofs[2])
        );

        vm.startPrank(addrs[3]);
        vm.deal(addrs[3], 1 ether);
        vm.expectRevert(MissingActivityCommitment.selector);
        saDiamond.activity().claim(
            subnetId,
            bottomUpCheckPeriod,
            Consensus.ValidatorData({validator: addrs[3], blocksCommitted: blocksMined[3]}),
            ActivityHelper.wrapBytes32Array(proofs[3])
        );

        // check
        assert(m.blocksCommitted(addrs[0]) == 1);
        assert(m.blocksCommitted(addrs[1]) == 2);
        assert(m.blocksCommitted(addrs[2]) == 0);
        assert(m.blocksCommitted(addrs[3]) == 0);
    }

    function testSubnetActor_ValidatorBatchClaimMiningReward_Works() public {
        ValidatorRewarderMap m = new ValidatorRewarderMap();
        {
            gatewayAddress = address(gatewayDiamond);

            Asset memory source = Asset({kind: AssetKind.Native, tokenAddress: address(0)});

            SubnetActorDiamond.ConstructorParams memory params = defaultSubnetActorParamsWith(
                gatewayAddress,
                SubnetID(ROOTNET_CHAINID, new address[](0)),
                source,
                AssetHelper.native()
            );
            params.validatorRewarder = address(m);
            params.minValidators = 2;
            params.permissionMode = PermissionMode.Federated;

            saDiamond = createSubnetActor(params);
        }

        SubnetID memory subnetId = SubnetID(ROOTNET_CHAINID, new address[](1));
        subnetId.route[0] = address(saDiamond);
        m.setSubnet(subnetId);

        (address[] memory addrs, uint256[] memory privKeys, bytes[] memory pubkeys) = TestUtils.newValidators(4);

        {
            uint256[] memory powers = new uint256[](4);
            powers[0] = 10000;
            powers[1] = 10000;
            powers[2] = 10000;
            powers[3] = 10000;
            saDiamond.manager().setFederatedPower(addrs, pubkeys, powers);
        }

        uint64[] memory blocksMined = new uint64[](addrs.length);

        blocksMined[0] = 1;
        blocksMined[1] = 2;

        (bytes32 activityRoot1, bytes32[][] memory proofs1) = MerkleTreeHelper.createMerkleProofsForConsensusActivity(
            addrs,
            blocksMined
        );

        (bytes32 activityRoot2, bytes32[][] memory proofs2) = MerkleTreeHelper.createMerkleProofsForConsensusActivity(
            addrs,
            blocksMined
        );

        confirmChange(addrs, privKeys, ActivityHelper.newCompressedActivityRollup(2, 3, activityRoot1));
        confirmChange(addrs, privKeys, ActivityHelper.newCompressedActivityRollup(2, 3, activityRoot2));

        vm.startPrank(addrs[0]);
        vm.deal(addrs[0], 1 ether);

        Consensus.ValidatorClaim[] memory claimProofs = new Consensus.ValidatorClaim[](2);
        uint64[] memory checkpointHeights = new uint64[](2);

        checkpointHeights[0] = uint64(gatewayDiamond.getter().bottomUpCheckPeriod());
        checkpointHeights[1] = uint64(gatewayDiamond.getter().bottomUpCheckPeriod()) * 2;

        claimProofs[0] = Consensus.ValidatorClaim({
            data: Consensus.ValidatorData({validator: addrs[0], blocksCommitted: blocksMined[0]}),
            proof: ActivityHelper.wrapBytes32Array(proofs1[0])
        });
        claimProofs[1] = Consensus.ValidatorClaim({
            data: Consensus.ValidatorData({validator: addrs[0], blocksCommitted: blocksMined[0]}),
            proof: ActivityHelper.wrapBytes32Array(proofs2[0])
        });

        saDiamond.activity().batchSubnetClaim(subnetId, checkpointHeights, claimProofs);

        // check
        assert(m.blocksCommitted(addrs[0]) == 2);
    }

    function testSubnetActor_ValidatorBatchClaimMiningReward_NoDoubleClaim() public {
        ValidatorRewarderMap m = new ValidatorRewarderMap();
        {
            gatewayAddress = address(gatewayDiamond);

            Asset memory source = Asset({kind: AssetKind.Native, tokenAddress: address(0)});

            SubnetActorDiamond.ConstructorParams memory params = defaultSubnetActorParamsWith(
                gatewayAddress,
                SubnetID(ROOTNET_CHAINID, new address[](0)),
                source,
                AssetHelper.native()
            );
            params.validatorRewarder = address(m);
            params.minValidators = 2;
            params.permissionMode = PermissionMode.Federated;

            saDiamond = createSubnetActor(params);
        }

        SubnetID memory subnetId = SubnetID(ROOTNET_CHAINID, new address[](1));
        subnetId.route[0] = address(saDiamond);
        m.setSubnet(subnetId);

        (address[] memory addrs, uint256[] memory privKeys, bytes[] memory pubkeys) = TestUtils.newValidators(4);

        {
            uint256[] memory powers = new uint256[](4);
            powers[0] = 10000;
            powers[1] = 10000;
            powers[2] = 10000;
            powers[3] = 10000;
            saDiamond.manager().setFederatedPower(addrs, pubkeys, powers);
        }

        uint64[] memory blocksMined = new uint64[](addrs.length);

        blocksMined[0] = 1;
        blocksMined[1] = 2;

        (bytes32 activityRoot1, bytes32[][] memory proofs1) = MerkleTreeHelper.createMerkleProofsForConsensusActivity(
            addrs,
            blocksMined
        );
        (bytes32 activityRoot2, bytes32[][] memory proofs2) = MerkleTreeHelper.createMerkleProofsForConsensusActivity(
            addrs,
            blocksMined
        );

        confirmChange(addrs, privKeys, ActivityHelper.newCompressedActivityRollup(2, 3, activityRoot1));
        confirmChange(addrs, privKeys, ActivityHelper.newCompressedActivityRollup(2, 3, activityRoot2));

        vm.startPrank(addrs[0]);
        vm.deal(addrs[0], 1 ether);

        Consensus.ValidatorClaim[] memory claimProofs = new Consensus.ValidatorClaim[](2);
        uint64[] memory heights = new uint64[](2);

        heights[0] = uint64(gatewayDiamond.getter().bottomUpCheckPeriod());
        heights[1] = uint64(gatewayDiamond.getter().bottomUpCheckPeriod());

        claimProofs[0] = Consensus.ValidatorClaim({
            data: Consensus.ValidatorData({validator: addrs[0], blocksCommitted: blocksMined[0]}),
            proof: ActivityHelper.wrapBytes32Array(proofs1[0])
        });
        claimProofs[1] = Consensus.ValidatorClaim({
            data: Consensus.ValidatorData({validator: addrs[0], blocksCommitted: blocksMined[0]}),
            proof: ActivityHelper.wrapBytes32Array(proofs2[0])
        });

        vm.expectRevert(ValidatorAlreadyClaimed.selector);
        saDiamond.activity().batchSubnetClaim(subnetId, heights, claimProofs);
    }

    function testGatewayDiamond_ValidatorBatchClaimERC20Reward_Works() public {
        MintingValidatorRewarder m = new MintingValidatorRewarder();
        {
            gatewayAddress = address(gatewayDiamond);

            Asset memory source = Asset({kind: AssetKind.Native, tokenAddress: address(0)});

            SubnetActorDiamond.ConstructorParams memory params = defaultSubnetActorParamsWith(
                gatewayAddress,
                SubnetID(ROOTNET_CHAINID, new address[](0)),
                source,
                AssetHelper.native()
            );
            params.validatorRewarder = address(m);
            params.minValidators = 2;
            params.permissionMode = PermissionMode.Federated;

            saDiamond = createSubnetActor(params);
        }

        SubnetID memory subnetId = SubnetID(ROOTNET_CHAINID, new address[](1));
        subnetId.route[0] = address(saDiamond);
        m.setSubnet(subnetId);

        (address[] memory addrs, uint256[] memory privKeys, bytes[] memory pubkeys) = TestUtils.newValidators(4);

        {
            uint256[] memory powers = new uint256[](4);
            powers[0] = 10000;
            powers[1] = 10000;
            powers[2] = 10000;
            powers[3] = 10000;
            saDiamond.manager().setFederatedPower(addrs, pubkeys, powers);
        }

        bytes[] memory metadata = new bytes[](addrs.length);
        uint64[] memory blocksMined = new uint64[](addrs.length);

        // assign extra metadata to validator 0
        // hardcode storageReward and uptimeReward to avoid stack too deep issues
        metadata[0] = abi.encode(uint256(100), uint256(10));

        blocksMined[0] = 11; // the first validator mined 11 blocks per checkpoint
        blocksMined[1] = 2; // the second validator mined 2 blocks per checkpoint

        (bytes32 activityRoot1, bytes32[][] memory proofs1) = MerkleTreeHelper.createMerkleProofsForConsensusActivity(
            addrs,
            blocksMined
        );

        (bytes32 activityRoot2, bytes32[][] memory proofs2) = MerkleTreeHelper.createMerkleProofsForConsensusActivity(
            addrs,
            blocksMined
        );

        // two checkpoints
        confirmChange(addrs, privKeys, ActivityHelper.newCompressedActivityRollup(2, 3, activityRoot1));
        confirmChange(addrs, privKeys, ActivityHelper.newCompressedActivityRollup(2, 3, activityRoot2));

        Consensus.ValidatorClaim[] memory claimProofs = new Consensus.ValidatorClaim[](2);
        uint64[] memory heights = new uint64[](2);

        heights[0] = uint64(gatewayDiamond.getter().bottomUpCheckPeriod());
        heights[1] = uint64(gatewayDiamond.getter().bottomUpCheckPeriod() * 2);

        // Validator 0 claims 11 blocks per checkpoint = 22 tokens
        claimProofs[0] = Consensus.ValidatorClaim({
            data: Consensus.ValidatorData({validator: addrs[0], blocksCommitted: blocksMined[0]}),
            proof: ActivityHelper.wrapBytes32Array(proofs1[0])
        });
        claimProofs[1] = Consensus.ValidatorClaim({
            data: Consensus.ValidatorData({validator: addrs[0], blocksCommitted: blocksMined[0]}),
            proof: ActivityHelper.wrapBytes32Array(proofs2[0])
        });

        vm.startPrank(addrs[0]);
        vm.deal(addrs[0], 1 ether);
        saDiamond.activity().batchSubnetClaim(subnetId, heights, claimProofs);

        // Validator 1 claims 2 blocks per checkpoint = 4 tokens
        claimProofs[0] = Consensus.ValidatorClaim({
            data: Consensus.ValidatorData({validator: addrs[1], blocksCommitted: blocksMined[1]}),
            proof: ActivityHelper.wrapBytes32Array(proofs1[1])
        });
        claimProofs[1] = Consensus.ValidatorClaim({
            data: Consensus.ValidatorData({validator: addrs[1], blocksCommitted: blocksMined[1]}),
            proof: ActivityHelper.wrapBytes32Array(proofs2[1])
        });

        vm.startPrank(addrs[1]);
        vm.deal(addrs[1], 1 ether);
        saDiamond.activity().batchSubnetClaim(subnetId, heights, claimProofs);

        // Assert
        assert(m.token().balanceOf(addrs[0]) == 22);
        assert(m.token().balanceOf(addrs[1]) == 4);
    }

    // -----------------------------------------------------------------------------------------------------------------
    // Tests for validator gater
    // -----------------------------------------------------------------------------------------------------------------
    function subnet_id(address baseRoute) internal view returns (SubnetID memory id) {
        address[] memory route = new address[](1);
        route[0] = baseRoute;

        SubnetID memory parent = saDiamond.getter().getParent();
        id = SubnetID({root: parent.root, route: route});
    }

    function testSubnetActorDiamond_ValidatorGater_set_works() public {
        saDiamond.manager().setValidatorGater(address(1));

        vm.prank(address(0));
        vm.expectRevert();
        saDiamond.manager().setValidatorGater(address(1));
    }

    function testSubnetActorDiamond_ValidatorGater_collateralLiftCycle() public {
        SubnetID memory id = subnet_id(address(saDiamond));
        address owner = address(1);

        vm.prank(owner);
        SubnetValidatorGater gater = new SubnetValidatorGater();
        vm.prank(owner);
        gater.setSubnet(id);

        saDiamond.manager().setValidatorGater(address(gater));

        (address validator, , bytes memory publicKey) = TestUtils.newValidator(100);

        vm.deal(validator, DEFAULT_MIN_VALIDATOR_STAKE * 3);
        vm.prank(validator);
        vm.expectRevert(ValidatorPowerChangeDenied.selector);
        saDiamond.manager().join{value: DEFAULT_MIN_VALIDATOR_STAKE}(publicKey, DEFAULT_MIN_VALIDATOR_STAKE);

        // now approve the join
        vm.prank(owner);
        gater.approve(validator, 2, DEFAULT_MIN_VALIDATOR_STAKE * 2);

        // should be able to join
        vm.prank(validator);
        saDiamond.manager().join{value: DEFAULT_MIN_VALIDATOR_STAKE}(publicKey, DEFAULT_MIN_VALIDATOR_STAKE);

        // add stake not allowed exceed allowed range
        vm.prank(validator);
        vm.expectRevert(ValidatorPowerChangeDenied.selector);
        saDiamond.manager().stake{value: DEFAULT_MIN_VALIDATOR_STAKE + 1}(DEFAULT_MIN_VALIDATOR_STAKE + 1);

        // add stake should be ok
        vm.prank(validator);
        saDiamond.manager().stake{value: DEFAULT_MIN_VALIDATOR_STAKE}(DEFAULT_MIN_VALIDATOR_STAKE);

        // unstake not allowed as below allowed range
        vm.prank(validator);
        vm.expectRevert(ValidatorPowerChangeDenied.selector);
        saDiamond.manager().unstake(DEFAULT_MIN_VALIDATOR_STAKE * 2 - 1);

        // unstake ok because within range
        vm.prank(validator);
        saDiamond.manager().unstake(DEFAULT_MIN_VALIDATOR_STAKE - 1);

        // leave not allowed as below allowed range
        vm.prank(validator);
        vm.expectRevert(ValidatorPowerChangeDenied.selector);
        saDiamond.manager().leave();

        // update allowed range
        vm.prank(owner);
        gater.approve(validator, 0, DEFAULT_MIN_VALIDATOR_STAKE * 2);

        // leave ok
        vm.prank(validator);
        saDiamond.manager().leave();
    }

    function testSubnetActorDiamond_ValidatorGater_federatedLiftCycle() public {
        address owner = address(1);

        vm.prank(owner);
        SubnetValidatorGater gater = new SubnetValidatorGater();

        gatewayAddress = address(gatewayDiamond);
        createSubnetActor(
            gatewayAddress,
            ConsensusType.Fendermint,
            DEFAULT_MIN_VALIDATOR_STAKE,
            DEFAULT_MIN_VALIDATORS,
            DEFAULT_CHECKPOINT_PERIOD,
            DEFAULT_MAJORITY_PERCENTAGE,
            PermissionMode.Federated,
            2,
            address(gater)
        );

        SubnetID memory id = subnet_id(address(saDiamond));
        vm.prank(owner);
        gater.setSubnet(id);

        (address[] memory validators, , bytes[] memory publicKeys) = TestUtils.newValidators(3);
        uint256[] memory powers = new uint256[](3);
        powers[0] = 10000;
        powers[1] = 20000;
        powers[2] = 5000; // we only have 2 active validators, validator 2 does not have enough power

        vm.expectRevert(ValidatorPowerChangeDenied.selector);
        saDiamond.manager().setFederatedPower(validators, publicKeys, powers);

        vm.prank(owner);
        gater.approve(validators[0], 0, powers[0]);
        vm.prank(owner);
        gater.approve(validators[1], 0, powers[1]);
        vm.prank(owner);
        gater.approve(validators[2], 0, powers[2]);

        saDiamond.manager().setFederatedPower(validators, publicKeys, powers);
    }

    function submitCheckpointInternal(
        BottomUpCheckpoint memory checkpoint,
        address[] memory validators,
        bytes[] memory signatures,
        uint256[] memory keys
    ) internal {
        bytes32 hash = keccak256(abi.encode(checkpoint));
        for (uint256 i = 0; i < 3; i++) {
            (uint8 v, bytes32 r, bytes32 s) = vm.sign(keys[i], hash);
            signatures[i] = abi.encodePacked(r, s, v);
        }

        saDiamond.checkpointer().submitCheckpoint(checkpoint, validators, signatures);
    }
}
