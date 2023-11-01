// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {SubnetActorDiamond} from "./SubnetActorDiamond.sol";
import {IDiamond} from "./interfaces/IDiamond.sol";
import {ReentrancyGuard} from "openzeppelin-contracts/security/ReentrancyGuard.sol";

contract SubnetRegistry is ReentrancyGuard {
    // solhint-disable-next-line var-name-mixedcase
    address public immutable GATEWAY;

    /// The getter and manager facet shared by diamond
    // solhint-disable-next-line var-name-mixedcase
    address public immutable SUBNET_GETTER_FACET;
    // solhint-disable-next-line var-name-mixedcase
    address public immutable SUBNET_MANAGER_FACET;

    /// The subnet getter facet functions selectors
    bytes4[] public subnetGetterSelectors;
    /// The subnet manager facet functions selectors
    bytes4[] public subnetManagerSelectors;

    /// @notice Mapping that tracks the deployed subnet actors per user.
    /// Key is the hash of Subnet ID, values are addresses.
    /// mapping owner => nonce => subnet
    mapping(address => mapping(uint64 => address)) public subnets;

    /// @notice Mapping that tracks the latest nonce of the deployed
    /// subnet for each user.
    /// owner => nonce
    mapping(address => uint64) public userNonces;

    /// @notice Event emitted when a new subnet is deployed.
    event SubnetDeployed(address subnetAddr);

    error FacetCannotBeZero();
    error WrongGateway();
    error CannotFindSubnet();
    error UnknownSubnet();
    error GatewayCannotBeZero();

    constructor(
        address _gateway,
        address _getterFacet,
        address _managerFacet,
        bytes4[] memory _subnetGetterSelectors,
        bytes4[] memory _subnetManagerSelectors
    ) {
        if (_gateway == address(0)) {
            revert GatewayCannotBeZero();
        }
        if (_getterFacet == address(0)) {
            revert FacetCannotBeZero();
        }
        if (_managerFacet == address(0)) {
            revert FacetCannotBeZero();
        }

        GATEWAY = _gateway;
        SUBNET_GETTER_FACET = _getterFacet;
        SUBNET_MANAGER_FACET = _managerFacet;

        subnetGetterSelectors = _subnetGetterSelectors;
        subnetManagerSelectors = _subnetManagerSelectors;
    }

    /// @notice Deploys a new subnet actor.
    /// @param _params The constructor params for Subnet Actor Diamond.
    function newSubnetActor(
        SubnetActorDiamond.ConstructorParams calldata _params
    ) external nonReentrant returns (address subnetAddr) {
        if (_params.ipcGatewayAddr != GATEWAY) {
            revert WrongGateway();
        }

        IDiamond.FacetCut[] memory diamondCut = new IDiamond.FacetCut[](2);

        // set the diamond cut for subnet getter
        diamondCut[0] = IDiamond.FacetCut({
            facetAddress: SUBNET_GETTER_FACET,
            action: IDiamond.FacetCutAction.Add,
            functionSelectors: subnetGetterSelectors
        });

        // set the diamond cut for subnet manager
        diamondCut[1] = IDiamond.FacetCut({
            facetAddress: SUBNET_MANAGER_FACET,
            action: IDiamond.FacetCutAction.Add,
            functionSelectors: subnetManagerSelectors
        });

        // slither-disable-next-line reentrancy-benign
        subnetAddr = address(new SubnetActorDiamond(diamondCut, _params));

        subnets[msg.sender][userNonces[msg.sender]] = subnetAddr;
        ++userNonces[msg.sender];

        emit SubnetDeployed(subnetAddr);
    }

    /// @notice Returns the address of the latest subnet actor deployed by a user
    function latestSubnetDeployed(address owner) external view returns (address subnet) {
        uint64 nonce = userNonces[owner];
        // need unchecked when nonce == 0 or else will underflow
        unchecked {
            nonce -= 1;
        }

        subnet = subnets[owner][nonce];
        if (subnet == address(0)) {
            revert CannotFindSubnet();
        }
    }

    /// @notice Returns the address of a subnet actor deployed for a specific nonce by a user
    function getSubnetDeployedByNonce(address owner, uint64 nonce) external view returns (address subnet) {
        subnet = subnets[owner][nonce];
        if (subnet == address(0)) {
            revert CannotFindSubnet();
        }
    }
}
