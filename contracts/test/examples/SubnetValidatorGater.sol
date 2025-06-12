// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {Test} from "forge-std/Test.sol";

import {SubnetID} from "../../contracts/structs/Subnet.sol";
import {SubnetValidatorGater, InvalidSubnet, ValidatorPowerChangeDenied, NotAuthorized} from "../../contracts/examples/SubnetValidatorGater.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

contract SubnetValidatorGaterTest is Test {
    function subnet_id(address baseRoute) internal pure returns (SubnetID memory id) {
        address[] memory route = new address[](1);
        route[0] = baseRoute;

        id = SubnetID({root: 0, route: route});
    }

    function test_gater_approve_works() public {
        SubnetID memory id = subnet_id(address(this));

        SubnetValidatorGater gater = new SubnetValidatorGater();
        gater.setSubnet(id);

        address validator = address(1);
        uint256 minPower = 100;
        uint256 maxPower = 200;
        gater.approve(validator, minPower, maxPower);
        require(gater.isAllow(validator, 110), "should allow");
    }

    function test_gater_approve_not_owner() public {
        SubnetID memory id = subnet_id(address(this));

        SubnetValidatorGater gater = new SubnetValidatorGater();
        gater.setSubnet(id);

        address validator = address(1);
        uint256 minPower = 100;
        uint256 maxPower = 200;

        vm.prank(validator);

        vm.expectRevert(abi.encodeWithSelector(Ownable.OwnableUnauthorizedAccount.selector, validator));
        gater.approve(validator, minPower, maxPower);

        require(!gater.isAllow(validator, 110), "should not allow");
    }

    function test_gater_intercept_ok() public {
        SubnetID memory id = subnet_id(address(this));

        SubnetValidatorGater gater = new SubnetValidatorGater();
        gater.setSubnet(id);

        address validator = address(1);
        uint256 minPower = 100;
        uint256 maxPower = 200;
        gater.approve(validator, minPower, maxPower);

        vm.prank(address(this));

        gater.interceptPowerDelta(id, validator, 0, 110);

        vm.expectRevert(ValidatorPowerChangeDenied.selector);
        gater.interceptPowerDelta(id, validator, 0, 210);
    }

    function test_gater_intercept_invalid_subnet() public {
        SubnetID memory id = subnet_id(address(this));

        SubnetValidatorGater gater = new SubnetValidatorGater();
        gater.setSubnet(id);

        address validator = address(1);
        uint256 minPower = 100;
        uint256 maxPower = 200;
        gater.approve(validator, minPower, maxPower);

        vm.prank(address(this));
        vm.expectRevert(InvalidSubnet.selector);

        gater.interceptPowerDelta(subnet_id(address(2)), validator, 0, 110);
    }

    function test_gater_intercept_not_authorized() public {
        SubnetID memory id = subnet_id(address(this));

        SubnetValidatorGater gater = new SubnetValidatorGater();
        gater.setSubnet(id);

        address validator = address(1);
        uint256 minPower = 100;
        uint256 maxPower = 200;
        gater.approve(validator, minPower, maxPower);

        vm.prank(validator);
        vm.expectRevert(abi.encodeWithSelector(NotAuthorized.selector, validator));

        gater.interceptPowerDelta(id, validator, 0, 110);
    }
}
