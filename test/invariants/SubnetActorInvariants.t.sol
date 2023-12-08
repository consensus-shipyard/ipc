// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import "../../src/errors/IPCErrors.sol";
import {StdInvariant, Test} from "forge-std/Test.sol";
import {TestUtils} from "../TestUtils.sol";
import {NumberContractFacetSeven, NumberContractFacetEight} from "../NumberContract.sol";
import {EMPTY_BYTES, METHOD_SEND, EMPTY_HASH} from "../../src/constants/Constants.sol";
import {ConsensusType} from "../../src/enums/ConsensusType.sol";
import {Status} from "../../src/enums/Status.sol";
import {CrossMsg, BottomUpCheckpoint, StorableMsg} from "../../src/structs/Checkpoint.sol";
import {FvmAddress} from "../../src/structs/FvmAddress.sol";
import {SubnetID, IPCAddress, Subnet, ValidatorInfo, Validator} from "../../src/structs/Subnet.sol";
import {StorableMsg} from "../../src/structs/Checkpoint.sol";
import {IERC165} from "../../src/interfaces/IERC165.sol";
import {IGateway} from "../../src/interfaces/IGateway.sol";
import {IDiamond} from "../../src/interfaces/IDiamond.sol";
import {IDiamondCut} from "../../src/interfaces/IDiamondCut.sol";
import {IDiamondLoupe} from "../../src/interfaces/IDiamondLoupe.sol";
import {FvmAddressHelper} from "../../src/lib/FvmAddressHelper.sol";
import {StorableMsgHelper} from "../../src/lib/StorableMsgHelper.sol";
import {SubnetIDHelper} from "../../src/lib/SubnetIDHelper.sol";
import {SubnetActorDiamond, FunctionNotFound} from "../../src/SubnetActorDiamond.sol";
import {GatewayDiamond} from "../../src/GatewayDiamond.sol";
import {GatewayGetterFacet} from "../../src/gateway/GatewayGetterFacet.sol";
import {GatewayMessengerFacet} from "../../src/gateway/GatewayMessengerFacet.sol";
import {GatewayManagerFacet} from "../../src/gateway/GatewayManagerFacet.sol";
import {GatewayRouterFacet} from "../../src/gateway/GatewayRouterFacet.sol";
import {SubnetActorHandler, ETH_SUPPLY} from "./handlers/SubnetActorHandler.sol";
import {SubnetActorManagerFacet} from "../../src/subnet/SubnetActorManagerFacet.sol";
import {SubnetActorGetterFacet} from "../../src/subnet/SubnetActorGetterFacet.sol";
import {DiamondCutFacet} from "../../src/diamond/DiamondCutFacet.sol";
import {DiamondLoupeFacet} from "../../src/diamond/DiamondLoupeFacet.sol";
import {FilAddress} from "fevmate/utils/FilAddress.sol";
import {LibStaking} from "../../src/lib/LibStaking.sol";
import {LibDiamond} from "../../src/lib/LibDiamond.sol";

import {SubnetActorManagerFacetMock} from "../mocks/SubnetActor.sol";

import {console} from "forge-std/console.sol";

contract SubnetActorInvariants is StdInvariant, Test {
    using SubnetIDHelper for SubnetID;
    using FilAddress for address;
    using FvmAddressHelper for FvmAddress;

    SubnetActorHandler private subnetActorHandler;

    address private constant DEFAULT_IPC_GATEWAY_ADDR = address(1024);
    uint64 private constant DEFAULT_CHECKPOINT_PERIOD = 10;
    uint256 private constant DEFAULT_MIN_ACTIVATION_COLLATERAL = 100 ether;
    uint64 private constant DEFAULT_MIN_VALIDATORS = 2;
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

    bytes4[] gwRouterSelectors;
    bytes4[] gwManagerSelectors;
    bytes4[] gwGetterSelectors;
    bytes4[] gwMessengerSelectors;

    SubnetActorDiamond saDiamond;
    SubnetActorManagerFacetMock saManager;
    SubnetActorGetterFacet saGetter;
    DiamondCutFacet cutFacet;
    DiamondLoupeFacet louper;

    GatewayDiamond gatewayDiamond;
    GatewayManagerFacet gwManager;
    GatewayGetterFacet gwGetter;
    GatewayRouterFacet gwRouter;
    GatewayMessengerFacet gwMessenger;

    constructor() {
        saGetterSelectors = TestUtils.generateSelectors(vm, "SubnetActorGetterFacet");
        saManagerSelectors = TestUtils.generateSelectors(vm, "SubnetActorManagerFacetMock");
        cutFacetSelectors = TestUtils.generateSelectors(vm, "DiamondCutFacet");
        louperSelectors = TestUtils.generateSelectors(vm, "DiamondLoupeFacet");

        gwRouterSelectors = TestUtils.generateSelectors(vm, "GatewayRouterFacet");
        gwGetterSelectors = TestUtils.generateSelectors(vm, "GatewayGetterFacet");
        gwManagerSelectors = TestUtils.generateSelectors(vm, "GatewayManagerFacet");
        gwMessengerSelectors = TestUtils.generateSelectors(vm, "GatewayMessengerFacet");
    }

    function setUp() public {
        GatewayDiamond.ConstructorParams memory gwConstructorParams = GatewayDiamond.ConstructorParams({
            networkName: SubnetID({root: ROOTNET_CHAINID, route: new address[](0)}),
            bottomUpCheckPeriod: DEFAULT_CHECKPOINT_PERIOD,
            msgFee: CROSS_MSG_FEE,
            minCollateral: DEFAULT_MIN_ACTIVATION_COLLATERAL,
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
            DEFAULT_MIN_ACTIVATION_COLLATERAL,
            DEFAULT_MIN_VALIDATORS,
            DEFAULT_CHECKPOINT_PERIOD,
            DEFAULT_MAJORITY_PERCENTAGE
        );

        subnetActorHandler = new SubnetActorHandler(saDiamond);

        bytes4[] memory fuzzSelectors = new bytes4[](4);
        fuzzSelectors[0] = SubnetActorHandler.join.selector;
        fuzzSelectors[1] = SubnetActorHandler.leave.selector;
        fuzzSelectors[2] = SubnetActorHandler.stake.selector;
        fuzzSelectors[3] = SubnetActorHandler.unstake.selector;

        targetSelector(FuzzSelector({addr: address(subnetActorHandler), selectors: fuzzSelectors}));
        targetContract(address(subnetActorHandler));
    }

    /// @notice The number of validators called `join` is equal to the number of total validators.
    function invariant_SA_01_total_validators_number_is_correct() public {
        assertEq(
            saGetter.getTotalValidatorsNumber(),
            subnetActorHandler.joinedValidatorsNumber(),
            "unexpected total validators number"
        );
    }

    /// @notice The stake of the subnet is the same from the GatewayActor and SubnetActor perspective.
    function invariant_SA_02_conservationOfETH() public {
        assertEq(
            ETH_SUPPLY,
            address(subnetActorHandler).balance + subnetActorHandler.ghost_stakedSum(),
            "subnet actor handler: unexpected stake"
        );
        assertEq(
            ETH_SUPPLY,
            address(subnetActorHandler).balance +
                saGetter.getTotalConfirmedCollateral() +
                subnetActorHandler.ghost_unstakedSum(),
            "subnet actor: unexpected stake"
        );

        if (saGetter.bootstrapped()) {
            SubnetID memory subnetId = gwGetter.getNetworkName().createSubnetId(address(saDiamond));
            Subnet memory subnet = gwGetter.subnets(subnetId.toHash());

            assertEq(
                subnetActorHandler.ghost_stakedSum() - subnetActorHandler.ghost_unstakedSum(),
                subnet.stake,
                "gateway actor: unexpected stake"
            );
        }
    }

    /// @notice The value resulting from all stake and unstake operations is equal to the total confirmed collateral.
    function invariant_SA_03_sum_of_stake_equals_collateral() public {
        assertEq(
            saGetter.getTotalConfirmedCollateral(),
            subnetActorHandler.ghost_stakedSum() - subnetActorHandler.ghost_unstakedSum()
        );
    }

    /// @notice Validator can withdraw all ETHs that it staked after leaving.
    function invariant_SA_04_validator_can_claim_collateral() public {
        address validator = subnetActorHandler.leave(0);
        if (validator == address(0)) {
            return;
        }
        if (!saGetter.bootstrapped()) {
            return;
        }

        uint256 subnetBalanceBefore = address(saDiamond).balance;
        uint256 balanceBefore = validator.balance;

        vm.prank(validator);
        saManager.claim();
        saManager.confirmNextChange();

        uint256 balanceAfter = validator.balance;
        uint256 subnetBalanceAfter = address(saDiamond).balance;

        assertEq(balanceAfter - balanceBefore, subnetBalanceBefore - subnetBalanceAfter, "unexpected claim amount");
    }

    /// @notice Total confirmed collateral equals sum of validator collaterals.
    function invariant_SA_05_total_collateral_equals_sum_of_validator_collaterals() public {
        uint256 sumOfCollaterals;
        address[] memory validators = subnetActorHandler.joinedValidators();
        uint256 n = validators.length;
        for (uint256 i; i < n; ++i) {
            sumOfCollaterals += saGetter.getTotalValidatorCollateral(validators[i]);
        }

        uint256 totalCollateral = saGetter.getTotalConfirmedCollateral();

        assertEq(sumOfCollaterals, totalCollateral, "unexpected sum of validators collateral");
    }

    /// @notice The collateral of the bootstrapped subnet is greater than minimum activation collateral.
    function iPoCnvariant_SA_05_collateral_is_sufficient() public {
        //console.log(saGetter.getTotalConfirmedCollateral());
        bool subnetBootstrapped = saGetter.bootstrapped();
        if (subnetBootstrapped) {
            assertGe(
                saGetter.getTotalConfirmedCollateral(),
                saGetter.minActivationCollateral(),
                "unexpected subnet collateral"
            );
        } else {
            assert(true);
        }
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

    function _assertDeploySubnetActor(
        address _ipcGatewayAddr,
        ConsensusType _consensus,
        uint256 _minActivationCollateral,
        uint64 _minValidators,
        uint64 _checkPeriod,
        uint8 _majorityPercentage
    ) public {
        SubnetID memory _parentId = SubnetID(ROOTNET_CHAINID, new address[](0));

        saManager = new SubnetActorManagerFacetMock();
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
                permissioned: false,
                minCrossMsgFee: CROSS_MSG_FEE
            })
        );

        saManager = SubnetActorManagerFacetMock(address(saDiamond));
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
