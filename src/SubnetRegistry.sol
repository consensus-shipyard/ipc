// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.19;

import {SubnetActor} from "./SubnetActor.sol";
import {SubnetID} from "./structs/Subnet.sol";
import {SubnetIDHelper} from "./lib/SubnetIDHelper.sol";

contract SubnetRegistry {
    using SubnetIDHelper for SubnetID;

    /// @notice Mapping that tracks the deployed subnet actors.
    /// Key is the hash of Subnet ID, values are addresses.
    mapping(bytes32 => address) public subnets;

    address public immutable gateway;

    /// @notice Event emitted when a new subnet is deployed.
    event SubnetDeployed(address subnetAddr, bytes32);

    error WrongGateway();
    error ZeroGatewayAddress();
    error UnknownSubnet();

    constructor(address _gateway) {
        if (_gateway == address(0)) {
            revert ZeroGatewayAddress();
        }
        gateway = _gateway;
    }

    function newSubnetActor(SubnetActor.ConstructParams calldata params) external returns (address subnetAddr) {
        if (params.ipcGatewayAddr != gateway) {
            revert WrongGateway();
        }

        subnetAddr = address(new SubnetActor(params));

        SubnetID memory id = params.parentId.createSubnetId(subnetAddr);

        bytes32 subnetHash = id.toHash();
        subnets[subnetHash] = subnetAddr;

        emit SubnetDeployed(subnetAddr, subnetHash);
    }

    function subnetAddress(bytes32 subnetHash) external view returns (address subnet) {
        subnet = subnets[subnetHash];
        if (subnet == address(0)) {
            revert UnknownSubnet();
        }
    }
}
