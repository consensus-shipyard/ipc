// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.18;

import "forge-std/Test.sol";

import "fevmate/utils/FilAddress.sol";
import "../src/lib/AccountHelper.sol";

contract DummyContract {
    using AccountHelper for address;

    bool public isAccount;

    constructor() {
        isAccount = address(this).isAccount();
    }
}

contract AccountHelperTest is Test {
    using AccountHelper for address;

    address constant ETH_ADDRESS = address(100);
    address constant BLS_ADDREESS = 0xfF000000000000000000000000000000bEefbEEf;

    function test_IsAccount_Fails_NonExistingAccount() public view {
        require(ETH_ADDRESS.isAccount() == false);
    }

    function test_IsAccount_Fails_BlsAccount() public view {
        require(BLS_ADDREESS.isAccount() == false);
    }

    function test_IsAccount_Works_ContractConstructor() public {
        DummyContract dc = new DummyContract();

        require(dc.isAccount() == true);
    }

    function test_IsAccount_Fails_ContractAccount() public {
        DummyContract dc = new DummyContract();

        require(address(dc).isAccount() == false);
    }

    function test_IsAccount_Works_EthAccount() public {
        activateAccount(ETH_ADDRESS);

        require(ETH_ADDRESS.isAccount() == true);
    }

    function test_IsAccount_Works_BlsAccount() public {
        activateAccount(BLS_ADDREESS);

        require(BLS_ADDREESS.isAccount() == true);
    }

    function test_IsSystemActor_True() public pure {
        require(FilAddress.SYSTEM_ACTOR.isSystemActor() == true);
    }

    function test_IsSystemActor_False() public pure {
        require(vm.addr(1234).isSystemActor() == false);
    }

    function activateAccount(address account) internal {
        vm.deal(account, 1 ether);
    }
}
