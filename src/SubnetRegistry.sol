// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.17;

import "./SubnetActor.sol";
import "./structs/Subnet.sol";
import "./lib/SubnetIDHelper.sol";

contract SubnetRegistry {
    using SubnetIDHelper for SubnetID;

    /// @notice Mapping that tracks the deployed subnet actors.
    /// Key is the hash of Subnet ID, values are addresses.
    mapping(bytes32 => address) public subnets;

    address public gateway;

    /// @notice Event emitted when a new subnet is deployed.
    event SubnetDeployed(address subnetAddr, SubnetID subnetId);

    error NotSameGateway();

    constructor(address _gateway) {
        gateway = _gateway;
    }

    function newSubnetActor(SubnetActor.ConstructParams calldata _params) external returns (address subnetAddr) {
        if (_params.ipcGatewayAddr != gateway) revert NotSameGateway();

        subnetAddr = address(new SubnetActor(_params));

        SubnetID memory id = _params.parentId.createSubnetId(subnetAddr);

        bytes32 subnetHash = id.toHash();
        subnets[subnetHash] = subnetAddr;

        emit SubnetDeployed(subnetAddr, id);
    }

    function subnetAddress(SubnetID calldata _subnetId) external view returns (address subnet) {
        bytes32 subnetHash = _subnetId.toHash();
        subnet = subnets[subnetHash];

        require(subnet != address(0), "Not exists");
    }
}
