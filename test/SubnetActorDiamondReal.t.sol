// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {Test} from "forge-std/Test.sol";
import {TestUtils} from "./TestUtils.sol";
import {console} from "forge-std/console.sol";
import "../src/errors/IPCErrors.sol";
import {TestUtils} from "./TestUtils.sol";
import {EMPTY_BYTES, METHOD_SEND, EMPTY_HASH} from "../src/constants/Constants.sol";
import {ConsensusType} from "../src/enums/ConsensusType.sol";
import {Status} from "../src/enums/Status.sol";
import {CrossMsg, BottomUpCheckpoint, StorableMsg} from "../src/structs/Checkpoint.sol";
import {FvmAddress} from "../src/structs/FvmAddress.sol";
import {SubnetID, IPCAddress, Subnet, ValidatorInfo, Validator} from "../src/structs/Subnet.sol";
import {StorableMsg} from "../src/structs/Checkpoint.sol";
import {IGateway} from "../src/interfaces/IGateway.sol";
import {IDiamond} from "../src/interfaces/IDiamond.sol";
import {IDiamondCut} from "../src/interfaces/IDiamondCut.sol";
import {FvmAddressHelper} from "../src/lib/FvmAddressHelper.sol";
import {CheckpointHelper} from "../src/lib/CheckpointHelper.sol";
import {StorableMsgHelper} from "../src/lib/StorableMsgHelper.sol";
import {SubnetIDHelper} from "../src/lib/SubnetIDHelper.sol";
import {SubnetActorDiamond} from "../src/SubnetActorDiamond.sol";
import {GatewayDiamond} from "../src/GatewayDiamond.sol";
import {GatewayGetterFacet} from "../src/gateway/GatewayGetterFacet.sol";
import {GatewayMessengerFacet} from "../src/gateway/GatewayMessengerFacet.sol";
import {GatewayManagerFacet} from "../src/gateway/GatewayManagerFacet.sol";
import {GatewayRouterFacet} from "../src/gateway/GatewayRouterFacet.sol";
import {SubnetActorManagerFacet} from "../src/subnet/SubnetActorManagerFacet.sol";
import {SubnetActorGetterFacet} from "../src/subnet/SubnetActorGetterFacet.sol";
import {FilAddress} from "fevmate/utils/FilAddress.sol";
import {LibStaking} from "../src/lib/LibStaking.sol";

import {DefaultGatewayMock} from "./subnetActorMock/DefaultGatewayMock.sol";
import {SubnetManagerTestUtil} from "./subnetActorMock/SubnetManagerTestUtil.sol";

contract SubnetActorDiamondRealTest is Test {
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

    bytes4[] gwRouterSelectors;
    bytes4[] gwManagerSelectors;
    bytes4[] gwGetterSelectors;
    bytes4[] gwMessengerSelectors;

    SubnetActorDiamond private saDiamond;
    SubnetManagerTestUtil private saManager;
    SubnetActorGetterFacet private saGetter;

    GatewayDiamond gatewayDiamond;
    GatewayManagerFacet gwManager;
    GatewayGetterFacet gwGetter;
    GatewayRouterFacet gwRouter;
    GatewayMessengerFacet gwMessenger;

    constructor() {
        saGetterSelectors = TestUtils.generateSelectors(vm, "SubnetActorGetterFacet");
        saManagerSelectors = TestUtils.generateSelectors(vm, "SubnetManagerTestUtil");

        gwRouterSelectors = TestUtils.generateSelectors(vm, "GatewayRouterFacet");
        gwGetterSelectors = TestUtils.generateSelectors(vm, "GatewayGetterFacet");
        gwManagerSelectors = TestUtils.generateSelectors(vm, "GatewayManagerFacet");
        gwMessengerSelectors = TestUtils.generateSelectors(vm, "GatewayMessengerFacet");
    }

    function createGatewayDiamond(GatewayDiamond.ConstructorParams memory params) public returns (GatewayDiamond) {
        gwRouter = new GatewayRouterFacet();
        gwManager = new GatewayManagerFacet();
        gwGetter = new GatewayGetterFacet();
        gwMessenger = new GatewayMessengerFacet();

        IDiamond.FacetCut[] memory diamondCut = new IDiamond.FacetCut[](4);

        diamondCut[0] = (
            IDiamond.FacetCut({
                facetAddress: address(gwRouter),
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: gwRouterSelectors
            })
        );

        diamondCut[1] = (
            IDiamond.FacetCut({
                facetAddress: address(gwManager),
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: gwManagerSelectors
            })
        );

        diamondCut[2] = (
            IDiamond.FacetCut({
                facetAddress: address(gwGetter),
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: gwGetterSelectors
            })
        );

        diamondCut[3] = (
            IDiamond.FacetCut({
                facetAddress: address(gwMessenger),
                action: IDiamond.FacetCutAction.Add,
                functionSelectors: gwMessengerSelectors
            })
        );

        gatewayDiamond = new GatewayDiamond(diamondCut, params);

        return gatewayDiamond;
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
        GatewayDiamond.ConstructorParams memory gwConstructorParams = GatewayDiamond.ConstructorParams({
            networkName: SubnetID({root: ROOTNET_CHAINID, route: new address[](0)}),
            bottomUpCheckPeriod: DEFAULT_CHECKPOINT_PERIOD,
            msgFee: CROSS_MSG_FEE,
            minCollateral: DEFAULT_MIN_VALIDATOR_STAKE,
            majorityPercentage: DEFAULT_MAJORITY_PERCENTAGE,
            activeValidatorsLimit: 4,
            genesisValidators: new Validator[](0)
        });

        gatewayDiamond = createGatewayDiamond(gwConstructorParams);

        gwGetter = GatewayGetterFacet(address(gatewayDiamond));
        gwManager = GatewayManagerFacet(address(gatewayDiamond));
        gwRouter = GatewayRouterFacet(address(gatewayDiamond));
        gwMessenger = GatewayMessengerFacet(address(gatewayDiamond));

        gatewayAddress = address(gatewayDiamond);

        _assertDeploySubnetActor(
            gatewayAddress,
            ConsensusType.Fendermint,
            DEFAULT_MIN_VALIDATOR_STAKE,
            DEFAULT_MIN_VALIDATORS,
            DEFAULT_CHECKPOINT_PERIOD,
            DEFAULT_MAJORITY_PERCENTAGE
        );
    }

    function testSubnetActorDiamondReal_BasicLifeCycle() public {
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

    function testSubnetActorDiamondReal_PreFund_works() public {
        (address validator1, bytes memory publicKey1) = TestUtils.deriveValidatorAddress(100);
        address preFunder = address(102);

        // total collateral in the gateway
        uint256 collateral = 0;
        uint256 fundAmount = 100;

        require(!saGetter.isActiveValidator(validator1), "active validator1");
        require(!saGetter.isWaitingValidator(validator1), "waiting validator1");

        // ======== Step. Join ======
        // pre-fund from validator and from pre-funder
        vm.startPrank(validator1);
        vm.deal(validator1, fundAmount);
        saManager.preFund{value: fundAmount}();
        vm.startPrank(preFunder);
        vm.deal(preFunder, fundAmount);
        saManager.preFund{value: fundAmount}();

        // initial validator joins
        vm.deal(validator1, DEFAULT_MIN_VALIDATOR_STAKE);

        vm.startPrank(validator1);
        saManager.join{value: DEFAULT_MIN_VALIDATOR_STAKE}(publicKey1);
        collateral = DEFAULT_MIN_VALIDATOR_STAKE;

        require(
            gatewayAddress.balance == collateral + 2 * fundAmount,
            "gw balance is incorrect after validator1 joining"
        );

        require(saGetter.genesisCircSupply() == 2 * fundAmount, "genesis circ supply not correct");
        (address[] memory genesisAddrs, ) = saGetter.genesisBalances();
        require(genesisAddrs.length == 2, "not two genesis addresses");

        // collateral confirmed immediately and network boostrapped
        ValidatorInfo memory v = saGetter.getValidator(validator1);
        require(v.totalCollateral == DEFAULT_MIN_VALIDATOR_STAKE, "total collateral not expected");
        require(v.confirmedCollateral == DEFAULT_MIN_VALIDATOR_STAKE, "confirmed collateral not equal to collateral");
        require(saGetter.isActiveValidator(validator1), "not active validator 1");
        require(!saGetter.isWaitingValidator(validator1), "waiting validator 1");
        TestUtils.ensureBytesEqual(v.metadata, publicKey1);
        require(saGetter.bootstrapped(), "subnet not bootstrapped");
        require(!saGetter.killed(), "subnet killed");
        require(saGetter.genesisValidators().length == 1, "not one validator in genesis");

        (uint64 nextConfigNum, uint64 startConfigNum) = saGetter.getConfigurationNumbers();
        require(nextConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER, "next config num not 1");
        require(startConfigNum == LibStaking.INITIAL_CONFIGURATION_NUMBER, "start config num not 1");

        vm.startPrank(preFunder);
        vm.expectRevert(SubnetAlreadyBootstrapped.selector);
        vm.deal(preFunder, fundAmount);
        saManager.preFund{value: fundAmount}();

        vm.stopPrank();
    }
}
