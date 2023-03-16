// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.7;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import "../src/Gateway.sol";
import "../src/lib/SubnetIDHelper.sol";
import "../src/lib/CheckpointHelper.sol";

contract GatewayDeploymentTest is Test {
    using SubnetIDHelper for SubnetID;
    using CheckpointHelper for Checkpoint;

    int64 constant DEFAULT_CHECKPOINT_PERIOD = 10;
    uint64 constant MIN_COLLATERAL_AMOUNT = 1 ether;
    uint64 constant MAX_NONCE = type(uint64).max;
    address constant PAYABLE_SUBNET_ADDRESS = address(10);

    Gateway gw;

    function setUp() public {
        address[] memory path = new address[](1);
        path[0] = address(0);
        gw = new Gateway(path, DEFAULT_CHECKPOINT_PERIOD);
    }

    function testDeployment(int64 checkpointPeriod) public {
        vm.assume(checkpointPeriod >= DEFAULT_CHECKPOINT_PERIOD);
        address[] memory path = new address[](1);
        path[0] = address(0);
        gw = new Gateway(path, checkpointPeriod);

        SubnetID memory networkName = gw.getNetworkName();

        require(networkName.isRoot());

        require(gw.minStake() == MIN_COLLATERAL_AMOUNT);
        require(gw.checkPeriod() == checkpointPeriod);
        require(gw.appliedBottomUpNonce() == MAX_NONCE);
    }

    function test_Register_Works_SingleSubnet(
        uint256 subnetCollateral,
        address subnetAddress
    ) public {
        vm.assume(subnetCollateral >= MIN_COLLATERAL_AMOUNT);
        vm.prank(subnetAddress);
        vm.deal(subnetAddress, subnetCollateral);

        registerSubnet(subnetCollateral, subnetAddress);

        require(gw.totalSubnets() == 1);
    }

    function test_Register_Works_MultipleSubnets(uint8 numberOfSubnets) public {
        vm.assume(numberOfSubnets > 0);

        for (uint i = 1; i <= numberOfSubnets; i++) {
            address subnetAddress = vm.addr(i);
            vm.prank(subnetAddress);
            vm.deal(subnetAddress, MIN_COLLATERAL_AMOUNT);

            registerSubnet(MIN_COLLATERAL_AMOUNT, subnetAddress);
        }

        require(gw.totalSubnets() == numberOfSubnets);
    }

    function test_Register_Fail_InsufficientCollateral(
        uint256 collateral
    ) public {
        vm.assume(collateral < MIN_COLLATERAL_AMOUNT);
        vm.expectRevert("call to register doesn't include enough funds");

        gw.register{value: collateral}();
    }

    function test_Register_Fail_SubnetAlreadyExists() public {
        registerSubnet(MIN_COLLATERAL_AMOUNT, address(this));

        vm.expectRevert("subnet is already registered");

        gw.register{value: MIN_COLLATERAL_AMOUNT}();
    }

    function test_AddStake_Works_SingleStaking(
        uint256 stakeAmount,
        uint256 registerAmount,
        address subnetAddress
    ) public {
        vm.assume(registerAmount >= MIN_COLLATERAL_AMOUNT);
        vm.assume(
            stakeAmount > 0 && stakeAmount < type(uint256).max - registerAmount
        );

        uint256 totalAmount = stakeAmount + registerAmount;

        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, totalAmount);

        registerSubnet(registerAmount, subnetAddress);
        addStake(stakeAmount, subnetAddress);

        (, uint totalStaked, , , ) = getSubnet(subnetAddress);

        require(totalStaked == totalAmount);
    }

    function test_AddStake_Works_MultipleStakings(uint8 numberOfStakes) public {
        vm.assume(numberOfStakes > 0);

        address subnetAddress = address(1);
        uint256 singleStakeAmount = 1 ether;
        uint256 registerAmount = MIN_COLLATERAL_AMOUNT;
        uint256 expectedStakedAmount = registerAmount;

        vm.startPrank(subnetAddress);
        vm.deal(
            subnetAddress,
            registerAmount + singleStakeAmount * numberOfStakes
        );

        registerSubnet(registerAmount, subnetAddress);

        for (uint i = 0; i < numberOfStakes; i++) {
            addStake(singleStakeAmount, subnetAddress);

            expectedStakedAmount += singleStakeAmount;
        }

        (, uint totalStake, , , ) = getSubnet(subnetAddress);

        require(totalStake == expectedStakedAmount);
    }

    function test_AddStake_Fail_ZeroAmount() public {
        registerSubnet(MIN_COLLATERAL_AMOUNT, address(this));

        vm.expectRevert("no stake to add");

        gw.addStake{value: 0}();
    }

    function test_AddStake_Fail_SubnetNotExists() public {
        vm.expectRevert("subnet is not registered");

        gw.addStake{value: 1}();
    }

    function test_ReleaseStake_Works_FullAmount(uint256 stakeAmount) public {
        address subnetAddress = PAYABLE_SUBNET_ADDRESS;
        uint256 registerAmount = MIN_COLLATERAL_AMOUNT;

        vm.assume(
            stakeAmount > 0 && stakeAmount < type(uint256).max - registerAmount
        );

        uint256 fullAmount = stakeAmount + registerAmount;

        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, fullAmount);

        registerSubnet(registerAmount, subnetAddress);
        addStake(stakeAmount, subnetAddress);

        gw.releaseStake(fullAmount);

        (, uint stake, , , Status status) = getSubnet(subnetAddress);

        require(stake == 0);
        require(status == Status.Inactive);
        require(subnetAddress.balance == fullAmount);
    }

    function test_ReleaseStake_Works_PartialAmount(
        uint256 partialAmount
    ) public {
        address subnetAddress = PAYABLE_SUBNET_ADDRESS;
        uint256 registerAmount = MIN_COLLATERAL_AMOUNT;

        vm.assume(
            partialAmount > registerAmount &&
                partialAmount < type(uint256).max - registerAmount
        );

        uint256 totalAmount = partialAmount + registerAmount;

        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, totalAmount);

        registerSubnet(registerAmount, subnetAddress);
        addStake(partialAmount, subnetAddress);

        gw.releaseStake(partialAmount);

        (, uint stake, , , Status status) = getSubnet(subnetAddress);

        require(stake == registerAmount);
        require(status == Status.Active);
        require(subnetAddress.balance == partialAmount);
    }

    function test_ReleaseStake_Fail_ZeroAmount() public {
        registerSubnet(MIN_COLLATERAL_AMOUNT, address(this));

        vm.expectRevert("no funds to release in params");

        gw.releaseStake(0);
    }

    function test_ReleaseStake_Fail_SubnetNotExists() public {
        vm.expectRevert("subnet is not registered");

        gw.releaseStake(1);
    }

    function test_ReleaseStake_Fail_InsufficientSubnetBalance(
        uint256 releaseAmount,
        uint256 subnetBalance,
        address subnetAddress
    ) public {
        vm.assume(subnetBalance > MIN_COLLATERAL_AMOUNT);
        vm.assume(releaseAmount > subnetBalance);
        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, releaseAmount);

        registerSubnet(subnetBalance, subnetAddress);

        vm.expectRevert("subnet actor not allowed to release so many funds");

        gw.releaseStake(releaseAmount);
    }

    function test_Kill_Works() public {
        address subnetAddress = PAYABLE_SUBNET_ADDRESS;

        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, MIN_COLLATERAL_AMOUNT);

        registerSubnet(MIN_COLLATERAL_AMOUNT, subnetAddress);

        require(subnetAddress.balance == 0);

        gw.kill();

        (
            SubnetID memory id,
            uint stake,
            uint nonce,
            uint circSupply,
            Status status
        ) = getSubnet(subnetAddress);

        require(id.toHash() == SubnetID(new address[](0)).toHash());
        require(stake == 0);
        require(nonce == 0);
        require(circSupply == 0);
        require(status == Status.Unset);
        require(gw.totalSubnets() == 0);
        require(subnetAddress.balance == MIN_COLLATERAL_AMOUNT);
    }

    function test_Kill_Fail_SubnetNotExists() public {
        vm.expectRevert("subnet is not registered");

        gw.kill();
    }

    function test_Kill_Fail_CircSupplyMoreThanZero() public {
        // TODO: implement once cross msg is implemented
    }

    function test_CommitChildCheck_Works(uint64 blockNumber) public {
        address subnetAddress = address(100);
        vm.assume(blockNumber < type(uint64).max / 2 - 9);

        vm.roll(blockNumber);

        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, MIN_COLLATERAL_AMOUNT);

        registerSubnet(MIN_COLLATERAL_AMOUNT, subnetAddress);
        
        Checkpoint memory checkpoint = createCheckpoint(subnetAddress, blockNumber + 9);

        Checkpoint memory commit = commitChildCheck(checkpoint);

        ChildCheck memory child = commit.data.children[0];

        require(child.checks.length == 1);
        require(child.checks[0] == checkpoint.toHash());
    }

    function test_CommitChildCheck_Works_SameSubnet(uint64 blockNumber) public {
        address subnetAddress = address(100);
        vm.assume(blockNumber < type(uint64).max / 2 - 11);
        vm.roll(blockNumber);
        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, MIN_COLLATERAL_AMOUNT);
        
        registerSubnet(MIN_COLLATERAL_AMOUNT, subnetAddress);

        Checkpoint memory checkpoint = createCheckpoint(subnetAddress, blockNumber + 9);
        Checkpoint memory commit = commitChildCheck(checkpoint);

        ChildCheck memory child = commit.data.children[0];

        require(child.checks.length == 1);
        require(child.checks[0] == checkpoint.toHash());

        vm.deal(subnetAddress, MIN_COLLATERAL_AMOUNT);

        Checkpoint memory checkpoint2 = createCheckpoint(subnetAddress, blockNumber + 11);

        checkpoint2.data.prevHash = checkpoint.toHash();
        Checkpoint memory commit2 = commitChildCheck(checkpoint2);
        
        ChildCheck memory child2 = commit2.data.children[0];
        
        require(child2.checks.length == 2);
        require(child2.checks[1] == checkpoint2.toHash());
    }
    
    function test_CommitChildCheck_Works_SecondSubnet(uint64 blockNumber) public {
        address firstSubnetAddress = address(100);
        address secondSubnetAddress = address(101);
        vm.assume(blockNumber < type(uint64).max / 2 - 9);

        vm.roll(blockNumber);
        vm.startPrank(firstSubnetAddress);
        vm.deal(firstSubnetAddress, MIN_COLLATERAL_AMOUNT);

        registerSubnet(MIN_COLLATERAL_AMOUNT, firstSubnetAddress);

        Checkpoint memory checkpoint = createCheckpoint(firstSubnetAddress, blockNumber + 9);
        Checkpoint memory commit = commitChildCheck(checkpoint);

        ChildCheck memory child = commit.data.children[0];

        require(child.checks.length == 1);
        require(child.checks[0] == checkpoint.toHash());

        vm.stopPrank();
        vm.startPrank(secondSubnetAddress);
        vm.deal(secondSubnetAddress, MIN_COLLATERAL_AMOUNT);

        registerSubnet(MIN_COLLATERAL_AMOUNT, secondSubnetAddress);

        Checkpoint memory checkpoint2 = createCheckpoint(secondSubnetAddress, blockNumber + 9);
        Checkpoint memory commit2 = commitChildCheck(checkpoint2);

        require(commit2.data.children.length == 2);

        ChildCheck memory child2 = commit2.data.children[1];

        require(child2.checks.length == 1);
        require(child2.checks[0] == checkpoint2.toHash());
    }


    function test_CommitChildCheck_Fail_NotConsistentWithPrevOne(uint64 blockNumber) public {
        address subnetAddress = address(100);
        vm.assume(blockNumber < type(uint64).max / 2 - 9);
        vm.roll(blockNumber);
        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, MIN_COLLATERAL_AMOUNT);

        registerSubnet(MIN_COLLATERAL_AMOUNT, subnetAddress);
        
        Checkpoint memory checkpoint = createCheckpoint(subnetAddress, blockNumber + 9);
        Checkpoint memory commit = commitChildCheck(checkpoint);

        ChildCheck memory child = commit.data.children[0];

        require(child.checks.length == 1);
        require(child.checks[0] == checkpoint.toHash());
        
        vm.expectRevert("previous checkpoint not consistent with previous one");

        checkpoint.data.prevHash = bytes32("0x1");
        gw.commitChildCheck(checkpoint);
    }

    function test_CommitChildCheck_Fail_CheckpointAlreadyExists(uint64 blockNumber) public {
        address subnetAddress = address(100);
        vm.assume(blockNumber < type(uint64).max / 2 - 9);
        vm.roll(blockNumber);
        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, MIN_COLLATERAL_AMOUNT);
        
        registerSubnet(MIN_COLLATERAL_AMOUNT, subnetAddress);

        Checkpoint memory checkpoint = createCheckpoint(subnetAddress, blockNumber + 9);
        Checkpoint memory commit = commitChildCheck(checkpoint);

        ChildCheck memory child = commit.data.children[0];

        require(child.checks.length == 1);
        require(child.checks[0] == checkpoint.toHash());

        vm.expectRevert("child checkpoint being committed already exists");

        gw.commitChildCheck(checkpoint);
    }

    function test_CommitChildCheck_Fail_InactiveSubnet(uint64 blockNumber) public {
        address subnetAddress = address(100);
        vm.assume(blockNumber < type(uint64).max / 2 - 9);
        vm.roll(blockNumber);
        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, MIN_COLLATERAL_AMOUNT);

        registerSubnet(MIN_COLLATERAL_AMOUNT, subnetAddress);

        gw.releaseStake(MIN_COLLATERAL_AMOUNT);

        Checkpoint memory checkpoint = createCheckpoint(subnetAddress, blockNumber + 9);
        
        vm.expectRevert("can't commit checkpoint for an inactive subnet");
        gw.commitChildCheck(checkpoint);
    }

    function test_CommitChildCheck_Fail_BelongsToThePast(uint64 blockNumber) public {
        address subnetAddress = address(100);
        vm.assume(blockNumber < type(uint64).max / 2 - 9);
        vm.roll(blockNumber);
        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, MIN_COLLATERAL_AMOUNT);

        registerSubnet(MIN_COLLATERAL_AMOUNT, subnetAddress);

        Checkpoint memory checkpoint = createCheckpoint(subnetAddress, blockNumber + 9);
        commitChildCheck(checkpoint);

        Checkpoint memory pastCheckpoint = createCheckpoint(subnetAddress, blockNumber + 8);

        vm.expectRevert("checkpoint being committed belongs to the past");
        gw.commitChildCheck(pastCheckpoint);
    }

    function test_CommitChildCheck_Fails_WrongSubnet(uint64 blockNumber) public {
        address subnetAddress = address(100);
        vm.assume(blockNumber < type(uint64).max / 2 - 9);
        vm.roll(blockNumber);
        vm.startPrank(subnetAddress);
        vm.deal(subnetAddress, MIN_COLLATERAL_AMOUNT);
        
        registerSubnet(MIN_COLLATERAL_AMOUNT, subnetAddress);

        Checkpoint memory checkpoint = createCheckpoint(subnetAddress, blockNumber + 9);
        
        address subnetAddressInvalid = address(101);

        vm.stopPrank();
        vm.startPrank(subnetAddressInvalid);

        vm.expectRevert("source in checkpoint doesn't belong to subnet");
        
        gw.commitChildCheck(checkpoint);
    }

    function commitChildCheck(Checkpoint memory commit) internal returns(Checkpoint memory){
        gw.commitChildCheck(commit);

        int64 epoch = (int64(uint64(block.number)) / gw.checkPeriod()) *
            gw.checkPeriod();

        (CheckData memory data, bytes memory signature) = gw.checkpoints(epoch);
        
        require(data.epoch == epoch);

        bool matchedSubnetId;
        for (uint childIndex = 0; childIndex < data.children.length; childIndex++) {
            if(
                data.children[childIndex].source.toHash() ==
                commit.data.source.toHash()
            ) {
                matchedSubnetId = true;
            }
        }
        require(matchedSubnetId == true);

        return Checkpoint(data, signature);
    }

    function createCheckpoint(address subnetAddress, uint64 blockNumber) internal view returns(Checkpoint memory) {
        SubnetID memory subnetId = gw.getNetworkName().createSubnetId(subnetAddress);

        CrossMsg[] memory crossMsgs = new CrossMsg[](0);
        ChildCheck[] memory children = new ChildCheck[](0);

        CrossMsgMeta memory crossMsgMeta = CrossMsgMeta({value: 0, nonce: 0, fee: 0, msgs: crossMsgs});
        CheckData memory data = CheckData({source: subnetId, tipSet: new bytes(0), epoch: int64(blockNumber), prevHash: bytes32(0), children: children, crossMsgs: crossMsgMeta });
        Checkpoint memory checkpoint = Checkpoint({data: data, signature: new bytes(0)});

        return checkpoint;
    }

    function addStake(uint stakeAmount, address subnetAddress) internal {
        uint256 balanceBefore = subnetAddress.balance;
        (, uint stakedBefore, , , ) = getSubnet(subnetAddress);

        gw.addStake{value: stakeAmount}();

        uint256 balanceAfter = subnetAddress.balance;
        (, uint stakedAfter, , , ) = getSubnet(subnetAddress);

        require(balanceAfter == balanceBefore - stakeAmount);
        require(stakedAfter == stakedBefore + stakeAmount);
    }

    function registerSubnet(
        uint256 collateral,
        address subnetAddress
    ) internal {
        gw.register{value: collateral}();

        (
            SubnetID memory id,
            uint stake,
            uint nonce,
            uint circSupply,
            Status status
        ) = getSubnet(subnetAddress);

        SubnetID memory parentNetwork = gw.getNetworkName();

        require(id.toHash() == parentNetwork.createSubnetId(subnetAddress).toHash());
        require(stake == collateral);
        require(nonce == 0);
        require(circSupply == 0);
        require(status == Status.Active);
    }

    function getSubnet(
        address subnetAddress
    ) internal view returns (SubnetID memory, uint, uint, uint, Status) {
        SubnetID memory subnetId = gw.getNetworkName().createSubnetId(subnetAddress);

        (
            SubnetID memory id,
            uint256 stake,
            uint256 nonce,
            uint256 circSupply,
            Status status,
        ) = gw.subnets(subnetId.toHash());

        return (id, stake, nonce, circSupply, status);
    }
}
