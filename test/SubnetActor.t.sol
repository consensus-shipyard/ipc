// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.7;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import "../src/SubnetActor.sol";

contract SubnetActorDeploymentTest is Test {
    address private constant IPC_GATEWAY_ADDR = address(1024);
    string private constant NETWORK_NAME = "test";

    SubnetActor sa;

    function testDeployment(string calldata _networkName, address _ipcGatewayAddr, uint256 _minValidatorStake, uint64 _minValidators, int64 _finalityTreshold, int64 _checkPeriod, bytes calldata _genesis) public {
        
        SubnetID memory parentId = SubnetID("/root", _ipcGatewayAddr);
        sa = new SubnetActor(parentId, _networkName, _ipcGatewayAddr, ConsensusType.Dummy, _minValidatorStake, _minValidators, _finalityTreshold, _checkPeriod, _genesis);
    
        require(keccak256(abi.encodePacked(sa.name())) == keccak256(abi.encodePacked(_networkName)));
        require(sa.ipcGatewayAddr() == _ipcGatewayAddr);
        require(sa.consensus() == ConsensusType.Dummy);
        require(sa.minValidatorStake() == _minValidatorStake);
        require(sa.minValidators() == _minValidators);
        require(sa.finalityThreshold() == _finalityTreshold);
        require(sa.checkPeriod() == _checkPeriod);
        require(keccak256(sa.genesis()) == keccak256(_genesis));
        (string memory parent, address actor) = sa.parentId();
        require(keccak256(abi.encodePacked(parent)) == keccak256(abi.encodePacked("/root")));
        require(actor == _ipcGatewayAddr);
    }
}