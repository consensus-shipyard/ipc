// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.7;

import "../src/Gateway.sol";
import "../src/SubnetActor.sol";
import "forge-std/Script.sol";

contract Deployer is Script {
    uint64 constant MIN_COLLATERAL_AMOUNT = 1 ether;
    uint64 private constant DEFAULT_MIN_VALIDATORS = 1;
    uint8 private constant DEFAULT_MAJORITY_PERCENTAGE = 70;
    uint64 constant DEFAULT_CHECKPOINT_PERIOD = 10;
    uint256 constant CROSS_MSG_FEE = 10 gwei;
    bytes private constant GENESIS = EMPTY_BYTES;
    address public constant ROOTNET_ADDRESS = address(0);
    bytes32 private constant DEFAULT_NETWORK_NAME = bytes32("test");
    uint64 private constant ROOTNET_CHAINID = 31415926;

    // add this to be excluded from coverage report
    function test() public {}

    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        vm.startBroadcast(deployerPrivateKey);

        address[] memory path = new address[](1);
        path[0] = ROOTNET_ADDRESS;

        Gateway.ConstructorParams memory constructorParams = Gateway.ConstructorParams({
            networkName: SubnetID({root: ROOTNET_CHAINID, route: path}),
            bottomUpCheckPeriod: DEFAULT_CHECKPOINT_PERIOD,
            topDownCheckPeriod: DEFAULT_CHECKPOINT_PERIOD,
            msgFee: CROSS_MSG_FEE,
            majorityPercentage: DEFAULT_MAJORITY_PERCENTAGE
        });
        Gateway gw = new Gateway(constructorParams);

        SubnetActor.ConstructParams memory subnetConstructorParams = SubnetActor.ConstructParams({
            parentId: SubnetID({root: ROOTNET_CHAINID, route: path}),
            name: DEFAULT_NETWORK_NAME,
            ipcGatewayAddr: address(gw),
            consensus: ConsensusType.Mir,
            minActivationCollateral: MIN_COLLATERAL_AMOUNT,
            minValidators: DEFAULT_MIN_VALIDATORS,
            bottomUpCheckPeriod: DEFAULT_CHECKPOINT_PERIOD,
            topDownCheckPeriod: DEFAULT_CHECKPOINT_PERIOD,
            majorityPercentage: DEFAULT_MAJORITY_PERCENTAGE,
            genesis: GENESIS
        });

        new SubnetActor(subnetConstructorParams);

        vm.stopBroadcast();
    }
}
