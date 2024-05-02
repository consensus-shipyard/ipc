// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

/// @notice A interface that indicated the implementing facet contains a or multiple genesis settings.
interface IGenesisComponent {
    /// @notice Returns the id of the component
    function id() external view returns(bytes4);

    /// @notice Returns the actual bytes of the genesis
    function genesis() external view returns(bytes memory);
}
