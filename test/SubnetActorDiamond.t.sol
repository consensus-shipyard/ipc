// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {Test} from "forge-std/Test.sol";
import {TestUtils} from "./TestUtils.sol";
import {console} from "forge-std/console.sol";
import "../src/errors/IPCErrors.sol";
import {TestUtils} from "./TestUtils.sol";
import {NumberContractFacetSeven, NumberContractFacetEight} from "./NumberContract.sol";
import {EMPTY_BYTES, METHOD_SEND, EMPTY_HASH} from "../src/constants/Constants.sol";
import {ConsensusType} from "../src/enums/ConsensusType.sol";
import {Status} from "../src/enums/Status.sol";
import {CrossMsg, BottomUpCheckpoint, StorableMsg} from "../src/structs/Checkpoint.sol";
import {FvmAddress} from "../src/structs/FvmAddress.sol";
import {SubnetID, IPCAddress, Subnet, ValidatorInfo} from "../src/structs/Subnet.sol";
import {StorableMsg} from "../src/structs/Checkpoint.sol";
import {IERC165} from "../src/interfaces/IERC165.sol";
import {IGateway} from "../src/interfaces/IGateway.sol";
import {IDiamond} from "../src/interfaces/IDiamond.sol";
import {IDiamondCut} from "../src/interfaces/IDiamondCut.sol";
import {IDiamondLoupe} from "../src/interfaces/IDiamondLoupe.sol";
import {FvmAddressHelper} from "../src/lib/FvmAddressHelper.sol";
import {CheckpointHelper} from "../src/lib/CheckpointHelper.sol";
import {StorableMsgHelper} from "../src/lib/StorableMsgHelper.sol";
import {SubnetIDHelper} from "../src/lib/SubnetIDHelper.sol";
import {SubnetActorDiamond, FunctionNotFound} from "../src/SubnetActorDiamond.sol";
import {SubnetActorManagerFacet} from "../src/subnet/SubnetActorManagerFacet.sol";
import {SubnetActorGetterFacet} from "../src/subnet/SubnetActorGetterFacet.sol";
import {DiamondCutFacet} from "../src/diamond/DiamondCutFacet.sol";
import {DiamondLoupeFacet} from "../src/diamond/DiamondLoupeFacet.sol";
import {LibDiamond} from "../src/lib/LibDiamond.sol";
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

    address gatewayAddress;
    IGateway gatewayContract;

    bytes4[] saGetterSelectors;
    bytes4[] saManagerSelectors;
    bytes4[] cutFacetSelectors;
    bytes4[] louperSelectors;

    SubnetActorDiamond saDiamond;
    SubnetManagerTestUtil saManager;
    SubnetActorGetterFacet saGetter;
    DiamondCutFacet cutFacet;
    DiamondLoupeFacet louper;

    constructor() {
        saGetterSelectors = TestUtils.generateSelectors(vm, "SubnetActorGetterFacet");
        saManagerSelectors = TestUtils.generateSelectors(vm, "SubnetManagerTestUtil");
        cutFacetSelectors = TestUtils.generateSelectors(vm, "DiamondCutFacet");
        louperSelectors = TestUtils.generateSelectors(vm, "DiamondLoupeFacet");
    }

    function createSubnetActorDiamondWithFaucets(
        SubnetActorDiamond.ConstructorParams memory params,
        address getterFaucet,
        address managerFaucet
    ) public returns (SubnetActorDiamond) {
        IDiamond.FacetCut[] memory diamondCut = new IDiamond.FacetCut[](4);

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
        diamondCut[2] = (
            IDiamond.FacetCut({
                facetAddress: address(cutFacet),
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: cutFacetSelectors
            })
        );

        diamondCut[3] = (
            IDiamond.FacetCut({
                facetAddress: address(louper),
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: louperSelectors
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

    function testSubnetActorDiamond_LoupeFunction() public view {
        require(louper.facets().length == 4, "unexpected length");
        require(louper.supportsInterface(type(IERC165).interfaceId) == true, "IERC165 not supported");
        require(louper.supportsInterface(type(IDiamondCut).interfaceId) == true, "IDiamondCut not supported");
        require(louper.supportsInterface(type(IDiamondLoupe).interfaceId) == true, "IDiamondLoupe not supported");
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

    function testSubnetActorDiamond_Join_Fail_InvalidPublicKeyLength() public {
        address validator = vm.addr(100);

        vm.deal(validator, 1 gwei);
        vm.prank(validator);
        vm.expectRevert(InvalidPublicKeyLength.selector);

        saManager.join{value: 10}(new bytes(64));
    }

    function testSubnetActorDiamond_Join_Fail_ZeroColalteral() public {
        (address validator, bytes memory publicKey) = TestUtils.deriveValidatorAddress(100);

        vm.deal(validator, 1 gwei);
        vm.prank(validator);
        vm.expectRevert(CollateralIsZero.selector);

        saManager.join(publicKey);
    }

    function testSubnetActorDiamond_Bootstrap_Node() public {
        (address validator, bytes memory publicKey) = TestUtils.deriveValidatorAddress(100);

        vm.deal(validator, DEFAULT_MIN_VALIDATOR_STAKE);
        vm.prank(validator);
        saManager.join{value: DEFAULT_MIN_VALIDATOR_STAKE}(publicKey);

        // validator adds empty node
        vm.prank(validator);
        vm.expectRevert(EmptyAddress.selector);
        saManager.addBootstrapNode("");

        // validator adds a node
        vm.prank(validator);
        saManager.addBootstrapNode("1.2.3.4");

        // not-validator adds a node
        vm.prank(vm.addr(200));
        vm.expectRevert(abi.encodeWithSelector(NotValidator.selector, vm.addr(200)));
        saManager.addBootstrapNode("3.4.5.6");

        string[] memory nodes = saGetter.getBootstrapNodes();
        require(nodes.length == 1, "it returns one node");
        require(
            keccak256(abi.encodePacked((nodes[0]))) == keccak256(abi.encodePacked(("1.2.3.4"))),
            "it returns correct address"
        );

        vm.prank(validator);
        saManager.leave();
        saManager.confirmChange(1);

        nodes = saGetter.getBootstrapNodes();
        require(nodes.length == 0, "no nodes");
    }

    function testSubnetActorDiamond_Leave_NotValidator() public {
        (address validator, ) = TestUtils.deriveValidatorAddress(100);

        // non-empty subnet can't be killed
        vm.startPrank(validator);
        vm.expectRevert(abi.encodeWithSelector(NotValidator.selector, validator));
        saManager.leave();
    }

    function testSubnetActorDiamond_Kill() public {
        (address validator, bytes memory publicKey) = TestUtils.deriveValidatorAddress(100);

        vm.deal(validator, DEFAULT_MIN_VALIDATOR_STAKE);
        vm.prank(validator);
        saManager.join{value: DEFAULT_MIN_VALIDATOR_STAKE}(publicKey);

        // non-empty subnet can't be killed
        vm.expectRevert(NotAllValidatorsHaveLeft.selector);
        saManager.kill();

        // leave the subnet and kill it
        vm.startPrank(validator);
        saManager.leave();
        saManager.confirmChange(1);

        // anyone can kill a subnet
        vm.startPrank(vm.addr(101));
        saManager.kill();
    }

    function testSubnetActorDiamond_Stake() public {
        (address validator, bytes memory publicKey) = TestUtils.deriveValidatorAddress(100);
        vm.deal(validator, 10 gwei);

        vm.prank(validator);
        vm.expectRevert(CollateralIsZero.selector);
        saManager.stake();

        vm.prank(validator);
        vm.expectRevert(NotStakedBefore.selector);
        saManager.stake{value: 10}();

        vm.prank(validator);
        saManager.join{value: 3}(publicKey);

        ValidatorInfo memory info = saGetter.getValidator(validator);
        require(info.totalCollateral == 3);
    }

    function testSubnetActorDiamond_crossMsgGetter() public view {
        CrossMsg[] memory msgs = new CrossMsg[](1);
        msgs[0] = CrossMsg({
            message: StorableMsg({
                from: IPCAddress({subnetId: saGetter.getParent(), rawAddress: FvmAddressHelper.from(address(this))}),
                to: IPCAddress({subnetId: saGetter.getParent(), rawAddress: FvmAddressHelper.from(address(this))}),
                value: CROSS_MSG_FEE + 1,
                nonce: 0,
                method: METHOD_SEND,
                params: new bytes(0),
                fee: CROSS_MSG_FEE
            }),
            wrapped: false
        });
        require(saGetter.crossMsgsHash(msgs) == keccak256(abi.encode(msgs)));
    }

    /// @notice Testing the basic join, stake, leave lifecycle of validators
    function testSubnetActorDiamond_BasicLifeCycle() public {
        (address validator1, bytes memory publicKey1) = TestUtils.deriveValidatorAddress(100);
        (address validator2, bytes memory publicKey2) = TestUtils.deriveValidatorAddress(101);

        // total collateral in the gateway
        uint256 collateral = 0;
        uint256 stake = 10;

        require(!saGetter.isActiveValidator(validator1), "active validator1");
        require(!saGetter.isWaitingValidator(validator1), "waiting validator1");

        require(!saGetter.isActiveValidator(validator2), "active validator2");
        require(!saGetter.isWaitingValidator(validator2), "waiting validator2");

        // ======== Step. Join ======
        // initial validator joins
        vm.deal(validator1, DEFAULT_MIN_VALIDATOR_STAKE);

        vm.startPrank(validator1);
        saManager.join{value: DEFAULT_MIN_VALIDATOR_STAKE}(publicKey1);
        collateral = DEFAULT_MIN_VALIDATOR_STAKE;

        require(gatewayAddress.balance == collateral, "gw balance is incorrect after validator1 joining");

        // collateral confirmed immediately and network boostrapped
        ValidatorInfo memory v = saGetter.getValidator(validator1);
        require(v.totalCollateral == DEFAULT_MIN_VALIDATOR_STAKE, "total collateral not expected");
        require(v.confirmedCollateral == DEFAULT_MIN_VALIDATOR_STAKE, "confirmed collateral not equal to collateral");
        require(saGetter.isActiveValidator(validator1), "not active validator 1");
        require(!saGetter.isWaitingValidator(validator1), "waiting validator 1");
        require(!saGetter.isActiveValidator(validator2), "active validator2");
        require(!saGetter.isWaitingValidator(validator2), "waiting validator2");
        TestUtils.ensureBytesEqual(v.metadata, publicKey1);
        require(saGetter.bootstrapped(), "subnet not bootstrapped");
        require(!saGetter.killed(), "subnet killed");
        require(saGetter.genesisValidators().length == 1, "not one validator in genesis");

        (uint64 nextConfigNum, uint64 startConfigNum) = saGetter.getConfigurationNumbers();
        require(nextConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER, "next config num not 1");
        require(startConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER, "start config num not 1");

        vm.stopPrank();

        // second validator joins
        vm.deal(validator2, DEFAULT_MIN_VALIDATOR_STAKE);

        vm.startPrank(validator2);
        // subnet bootstrapped and should go through queue
        saManager.join{value: DEFAULT_MIN_VALIDATOR_STAKE}(publicKey2);

        // collateral not confirmed yet
        v = saGetter.getValidator(validator2);
        require(gatewayAddress.balance == collateral, "gw balance is incorrect after validator2 joining");
        require(v.totalCollateral == DEFAULT_MIN_VALIDATOR_STAKE, "total collateral not expected");
        require(v.confirmedCollateral == 0, "confirmed collateral not equal to collateral");
        require(saGetter.isActiveValidator(validator1), "not active validator 1");
        require(!saGetter.isWaitingValidator(validator1), "waiting validator 1");
        require(!saGetter.isActiveValidator(validator2), "active validator2");
        require(!saGetter.isWaitingValidator(validator2), "waiting validator2");
        TestUtils.ensureBytesEqual(v.metadata, new bytes(0));

        (nextConfigNum, startConfigNum) = saGetter.getConfigurationNumbers();
        // join will update the metadata, incr by 1
        // join will call deposit, incr by 1
        require(nextConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER + 2, "next config num not 3");
        require(startConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER, "start config num not 1");
        vm.stopPrank();

        // ======== Step. Confirm join operation ======
        collateral += DEFAULT_MIN_VALIDATOR_STAKE;
        saManager.confirmChange(LibStaking.INITIAL_CONFIGURATION_NUMBER + 1);
        require(gatewayAddress.balance == collateral, "gw balance is incorrect after validator2 joining");

        v = saGetter.getValidator(validator2);
        require(v.totalCollateral == DEFAULT_MIN_VALIDATOR_STAKE, "total collateral not expected after confirm join");
        require(
            v.confirmedCollateral == DEFAULT_MIN_VALIDATOR_STAKE,
            "confirmed collateral not expected after confirm join"
        );
        require(saGetter.isActiveValidator(validator1), "not active validator1");
        require(!saGetter.isWaitingValidator(validator1), "waiting validator1");
        require(saGetter.isActiveValidator(validator2), "not active validator2");
        require(!saGetter.isWaitingValidator(validator2), "waiting validator2");

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
        vm.deal(validator1, stake);

        saManager.stake{value: stake}();

        v = saGetter.getValidator(validator1);
        require(v.totalCollateral == DEFAULT_MIN_VALIDATOR_STAKE + stake, "total collateral not expected after stake");
        require(v.confirmedCollateral == DEFAULT_MIN_VALIDATOR_STAKE, "confirmed collateral not 0 after stake");
        require(gatewayAddress.balance == collateral, "gw balance is incorrect after validator1 stakes more");

        (nextConfigNum, startConfigNum) = saGetter.getConfigurationNumbers();
        require(nextConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER + 3, "next config num not 4 after stake");
        require(startConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER + 2, "start config num not 3 after stake");

        vm.stopPrank();

        // ======== Step. Confirm stake operation ======
        collateral += stake;
        saManager.confirmChange(LibStaking.INITIAL_CONFIGURATION_NUMBER + 2);
        require(gatewayAddress.balance == collateral, "gw balance is incorrect after confirm stake");

        v = saGetter.getValidator(validator1);
        require(
            v.totalCollateral == DEFAULT_MIN_VALIDATOR_STAKE + stake,
            "total collateral not expected after confirm stake"
        );
        require(
            v.confirmedCollateral == DEFAULT_MIN_VALIDATOR_STAKE + stake,
            "confirmed collateral not expected after confirm stake"
        );

        v = saGetter.getValidator(validator2);
        require(v.totalCollateral == DEFAULT_MIN_VALIDATOR_STAKE, "total collateral not expected after confirm stake");
        require(
            v.confirmedCollateral == DEFAULT_MIN_VALIDATOR_STAKE,
            "confirmed collateral not expected after confirm stake"
        );

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
        require(
            v.confirmedCollateral == DEFAULT_MIN_VALIDATOR_STAKE + stake,
            "confirmed collateral incorrect after confirm leave"
        );
        require(gatewayAddress.balance == collateral, "gw balance is incorrect after validator 1 leaving");

        (nextConfigNum, startConfigNum) = saGetter.getConfigurationNumbers();
        require(
            nextConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER + 4,
            "next config num not 5 after confirm leave"
        );
        require(
            startConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER + 3,
            "start config num not 4 after confirm leave"
        );
        require(saGetter.isActiveValidator(validator1), "not active validator 1");
        require(saGetter.isActiveValidator(validator2), "not active validator 2");

        vm.stopPrank();

        // ======== Step. Confirm leave ======
        saManager.confirmChange(LibStaking.INITIAL_CONFIGURATION_NUMBER + 3);
        collateral -= (DEFAULT_MIN_VALIDATOR_STAKE + stake);
        require(gatewayAddress.balance == collateral, "gw balance is incorrect after confirming validator 1 leaving");

        v = saGetter.getValidator(validator1);
        require(v.totalCollateral == 0, "total collateral not 0 after confirm leave");
        require(v.confirmedCollateral == 0, "confirmed collateral not 0 after confirm leave");

        (nextConfigNum, startConfigNum) = saGetter.getConfigurationNumbers();
        require(
            nextConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER + 4,
            "next config num not 5 after confirm leave"
        );
        require(
            startConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER + 4,
            "start config num not 5 after confirm leave"
        );
        require(!saGetter.isActiveValidator(validator1), "active validator 1");
        require(saGetter.isActiveValidator(validator2), "not active validator 2");

        // ======== Step. Claim collateral ======
        uint256 b1 = validator1.balance;
        vm.startPrank(validator1);
        saManager.claim();
        uint256 b2 = validator1.balance;
        require(b2 - b1 == DEFAULT_MIN_VALIDATOR_STAKE + stake, "collateral not received");
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
            saManager.join{value: 10}(pubKeys[i]);
        }

        saManager.validateActiveQuorumSignatures(validators, hash, signatures);
    }

    function testSubnetActorDiamond_validateActiveQuorumSignatures_InvalidSignature() public {
        (uint256[] memory keys, address[] memory validators, ) = TestUtils.getThreeValidators(vm);
        bytes[] memory pubKeys = new bytes[](3);
        bytes[] memory signatures = new bytes[](3);

        bytes32 hash = keccak256(abi.encodePacked("test"));
        bytes32 hash0 = keccak256(abi.encodePacked("test1"));

        for (uint256 i = 0; i < 3; i++) {
            (uint8 v, bytes32 r, bytes32 s) = vm.sign(keys[i], hash);
            signatures[i] = abi.encodePacked(r, s, v);
            pubKeys[i] = TestUtils.deriveValidatorPubKeyBytes(keys[i]);
            vm.deal(validators[i], 10 gwei);
            vm.prank(validators[i]);
            saManager.join{value: 10}(pubKeys[i]);
        }

        vm.expectRevert(abi.encodeWithSelector(InvalidSignatureErr.selector, 4));
        saManager.validateActiveQuorumSignatures(validators, hash0, signatures);
    }

    function testSubnetActorDiamond_submitCheckpoint_basic() public {
        (uint256[] memory keys, address[] memory validators, ) = TestUtils.getThreeValidators(vm);
        bytes[] memory pubKeys = new bytes[](3);
        bytes[] memory signatures = new bytes[](3);

        for (uint256 i = 0; i < 3; i++) {
            vm.deal(validators[i], 10 gwei);
            pubKeys[i] = TestUtils.deriveValidatorPubKeyBytes(keys[i]);
            vm.prank(validators[i]);
            saManager.join{value: 10}(pubKeys[i]);
        }

        CrossMsg memory crossMsg = CrossMsg({
            message: StorableMsg({
                from: IPCAddress({subnetId: saGetter.getParent(), rawAddress: FvmAddressHelper.from(address(this))}),
                to: IPCAddress({subnetId: saGetter.getParent(), rawAddress: FvmAddressHelper.from(address(this))}),
                value: CROSS_MSG_FEE + 1,
                nonce: 0,
                method: METHOD_SEND,
                params: new bytes(0),
                fee: CROSS_MSG_FEE
            }),
            wrapped: false
        });
        CrossMsg[] memory msgs = new CrossMsg[](1);
        msgs[0] = crossMsg;

        BottomUpCheckpoint memory checkpoint = BottomUpCheckpoint({
            subnetID: saGetter.getParent(),
            blockHeight: saGetter.bottomUpCheckPeriod(),
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 0,
            crossMessagesHash: keccak256(abi.encode(msgs))
        });

        BottomUpCheckpoint memory checkpointWithIncorrectHeight = BottomUpCheckpoint({
            subnetID: saGetter.getParent(),
            blockHeight: 1,
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 0,
            crossMessagesHash: keccak256(abi.encode(msgs))
        });

        BottomUpCheckpoint memory checkpointWithIncorrectHash = BottomUpCheckpoint({
            subnetID: saGetter.getParent(),
            blockHeight: saGetter.bottomUpCheckPeriod(),
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 0,
            crossMessagesHash: keccak256(abi.encode("1"))
        });

        bytes32 hash = keccak256(abi.encode(checkpoint));

        for (uint256 i = 0; i < 3; i++) {
            (uint8 v, bytes32 r, bytes32 s) = vm.sign(keys[i], hash);
            signatures[i] = abi.encodePacked(r, s, v);
        }

        vm.expectRevert(InvalidCheckpointEpoch.selector);
        vm.prank(validators[0]);
        saManager.submitCheckpoint(checkpointWithIncorrectHeight, msgs, validators, signatures);

        vm.expectRevert(InvalidCheckpointMessagesHash.selector);
        vm.prank(validators[0]);
        saManager.submitCheckpoint(checkpointWithIncorrectHash, msgs, validators, signatures);

        vm.expectCall(gatewayAddress, abi.encodeCall(IGateway.commitBottomUpCheckpoint, (checkpoint, msgs)), 1);
        vm.prank(validators[0]);
        saManager.submitCheckpoint(checkpoint, msgs, validators, signatures);
        require(saGetter.hasSubmittedInLastBottomUpCheckpointHeight(validators[0]), "validator rewarded");
        require(
            saGetter.lastBottomUpCheckpointHeight() == saGetter.bottomUpCheckPeriod(),
            " checkpoint height correct"
        );

        vm.prank(validators[1]);
        saManager.submitCheckpoint(checkpoint, msgs, validators, signatures);
        require(saGetter.hasSubmittedInLastBottomUpCheckpointHeight(validators[1]), "validator rewarded");
        require(
            saGetter.lastBottomUpCheckpointHeight() == saGetter.bottomUpCheckPeriod(),
            " checkpoint height correct"
        );

        (bool exists, BottomUpCheckpoint memory recvCheckpoint) = saGetter.bottomUpCheckpointAtEpoch(
            saGetter.bottomUpCheckPeriod()
        );
        require(exists, "checkpoint does not exist");
        require(hash == keccak256(abi.encode(recvCheckpoint)), "checkpoint hashes are not the same");

        bytes32 recvHash;
        (exists, recvHash) = saGetter.bottomUpCheckpointHashAtEpoch(saGetter.bottomUpCheckPeriod());
        require(exists, "checkpoint does not exist");
        require(hash == recvHash, "hashes are not the same");
    }

    function testSubnetActorDiamond_submitCheckpointWithReward() public {
        (uint256[] memory keys, address[] memory validators, ) = TestUtils.getThreeValidators(vm);
        bytes[] memory pubKeys = new bytes[](3);
        bytes[] memory signatures = new bytes[](3);

        for (uint256 i = 0; i < 3; i++) {
            vm.deal(validators[i], 10 gwei);
            pubKeys[i] = TestUtils.deriveValidatorPubKeyBytes(keys[i]);
            vm.prank(validators[i]);
            saManager.join{value: 10}(pubKeys[i]);
        }

        // send the first checkpoint
        CrossMsg memory crossMsg = CrossMsg({
            message: StorableMsg({
                from: IPCAddress({subnetId: saGetter.getParent(), rawAddress: FvmAddressHelper.from(address(this))}),
                to: IPCAddress({subnetId: saGetter.getParent(), rawAddress: FvmAddressHelper.from(address(this))}),
                value: CROSS_MSG_FEE + 1,
                nonce: 0,
                method: METHOD_SEND,
                params: new bytes(0),
                fee: CROSS_MSG_FEE
            }),
            wrapped: false
        });
        CrossMsg[] memory msgs = new CrossMsg[](1);
        msgs[0] = crossMsg;

        BottomUpCheckpoint memory checkpoint = BottomUpCheckpoint({
            subnetID: saGetter.getParent(),
            blockHeight: saGetter.bottomUpCheckPeriod(),
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 0,
            crossMessagesHash: keccak256(abi.encode(msgs))
        });

        bytes32 hash = keccak256(abi.encode(checkpoint));

        for (uint256 i = 0; i < 3; i++) {
            (uint8 v, bytes32 r, bytes32 s) = vm.sign(keys[i], hash);
            signatures[i] = abi.encodePacked(r, s, v);
        }

        vm.expectCall(gatewayAddress, abi.encodeCall(IGateway.commitBottomUpCheckpoint, (checkpoint, msgs)), 1);
        vm.prank(validators[0]);
        saManager.submitCheckpoint(checkpoint, msgs, validators, signatures);

        require(saGetter.hasSubmittedInLastBottomUpCheckpointHeight(validators[0]), "validator rewarded");
        require(
            saGetter.lastBottomUpCheckpointHeight() == saGetter.bottomUpCheckPeriod(),
            " checkpoint height correct"
        );

        require(saGetter.getRelayerReward(validators[0]) == 0, "there is a reward");
        vm.startPrank(gatewayAddress);
        saManager.distributeRewardToRelayers(saGetter.bottomUpCheckPeriod(), 10);
        require(saGetter.getRelayerReward(validators[0]) == 0, "there is the reward for block 0");

        // send the second checkpoint
        checkpoint = BottomUpCheckpoint({
            subnetID: saGetter.getParent(),
            blockHeight: 2 * saGetter.bottomUpCheckPeriod(),
            blockHash: keccak256("block2"),
            nextConfigurationNumber: 0,
            crossMessagesHash: keccak256(abi.encode(msgs))
        });

        hash = keccak256(abi.encode(checkpoint));

        for (uint256 i = 0; i < 3; i++) {
            (uint8 v, bytes32 r, bytes32 s) = vm.sign(keys[i], hash);
            signatures[i] = abi.encodePacked(r, s, v);
        }

        saManager.submitCheckpoint(checkpoint, msgs, validators, signatures);
        vm.startPrank(gatewayAddress);
        saManager.distributeRewardToRelayers(2 * saGetter.bottomUpCheckPeriod(), 0);
        require(saGetter.getRelayerReward(validators[0]) == 0, "reword more than 0");

        saManager.submitCheckpoint(checkpoint, msgs, validators, signatures);
        vm.startPrank(gatewayAddress);
        saManager.distributeRewardToRelayers(2 * saGetter.bottomUpCheckPeriod(), 10);
        uint256 validator1Reward = saGetter.getRelayerReward(validators[0]);
        require(validator1Reward == 10, "there is no reward for block 1");

        uint256 b1 = validators[0].balance;
        vm.startPrank(validators[0]);
        saManager.claimRewardForRelayer();
        uint256 b2 = validators[0].balance;
        require(b2 - b1 == validator1Reward, "reward received");
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
        bytes4[] memory ncGetterSelectors = TestUtils.generateSelectors(vm, "NumberContractFacetSeven");

        saDiamondCut[0] = (
            IDiamond.FacetCut({
                facetAddress: address(ncFacetA),
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: ncGetterSelectors
            })
        );
        //test that other user cannot call diamondcut to add function
        vm.prank(0x1234567890123456789012345678901234567890);
        vm.expectRevert(LibDiamond.NotOwner.selector);
        saDiamondCutter.diamondCut(saDiamondCut, address(0), new bytes(0));

        saDiamondCutter.diamondCut(saDiamondCut, address(0), new bytes(0));

        NumberContractFacetSeven saNumberContract = NumberContractFacetSeven(address(saDiamond));
        assert(saNumberContract.getNum() == 7);

        ncGetterSelectors = TestUtils.generateSelectors(vm, "NumberContractFacetEight");
        saDiamondCut[0] = (
            IDiamond.FacetCut({
                facetAddress: address(ncFacetB),
                action: IDiamond.FacetCutAction.Replace,
                functionSelectors: ncGetterSelectors
            })
        );

        //test that other user cannot call diamondcut to replace function
        vm.prank(0x1234567890123456789012345678901234567890);
        vm.expectRevert(LibDiamond.NotOwner.selector);
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
        vm.expectRevert(LibDiamond.NotOwner.selector);
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
        saManager.unstake(0);

        vm.expectRevert(abi.encodeWithSelector(NotValidator.selector, validator));
        vm.prank(validator);
        saManager.unstake(10);

        vm.deal(validator, DEFAULT_MIN_VALIDATOR_STAKE);
        vm.prank(validator);
        saManager.join{value: DEFAULT_MIN_VALIDATOR_STAKE}(publicKey);
        require(
            saGetter.getValidator(validator).totalCollateral == DEFAULT_MIN_VALIDATOR_STAKE,
            "initial collateral correct"
        );

        vm.expectRevert(NotEnoughCollateral.selector);
        vm.prank(validator);
        saManager.unstake(DEFAULT_MIN_VALIDATOR_STAKE + 100);

        vm.expectRevert(NotEnoughCollateral.selector);
        vm.prank(validator);
        saManager.unstake(DEFAULT_MIN_VALIDATOR_STAKE);

        vm.prank(validator);
        saManager.unstake(5);
        require(
            saGetter.getValidator(validator).totalCollateral == DEFAULT_MIN_VALIDATOR_STAKE - 5,
            "collateral correct after unstaking"
        );
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
        cutFacet = new DiamondCutFacet();
        louper = new DiamondLoupeFacet();

        IDiamond.FacetCut[] memory diamondCut = new IDiamond.FacetCut[](4);

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

        diamondCut[2] = (
            IDiamond.FacetCut({
                facetAddress: address(cutFacet),
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: cutFacetSelectors
            })
        );

        diamondCut[3] = (
            IDiamond.FacetCut({
                facetAddress: address(louper),
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: louperSelectors
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
        cutFacet = DiamondCutFacet(address(saDiamond));
        louper = DiamondLoupeFacet(address(saDiamond));

        require(saGetter.ipcGatewayAddr() == _ipcGatewayAddr, "saGetter.ipcGatewayAddr() == _ipcGatewayAddr");
        require(
            saGetter.minActivationCollateral() == _minActivationCollateral,
            "saGetter.minActivationCollateral() == _minActivationCollateral"
        );
        require(saGetter.minValidators() == _minValidators, "saGetter.minValidators() == _minValidators");
        require(saGetter.consensus() == _consensus, "consensus");
        require(saGetter.getParent().equals(_parentId), "parent");
        require(saGetter.activeValidatorsLimit() == 100, "activeValidatorsLimit");
        require(saGetter.powerScale() == 12, "powerscale");
        require(saGetter.minCrossMsgFee() == CROSS_MSG_FEE, "cross-msg fee");
        require(saGetter.bottomUpCheckPeriod() == _checkPeriod, "bottom-up period");
        require(saGetter.majorityPercentage() == _majorityPercentage, "majority percentage");
        require(
            saGetter.getParent().toHash() == _parentId.toHash(),
            "parent.toHash() == SubnetID({root: ROOTNET_CHAINID, route: path}).toHash()"
        );
    }
}
