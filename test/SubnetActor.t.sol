// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.7;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import "../src/SubnetActor.sol";
import "../src/Gateway.sol";
import "../src/enums/Status.sol";
import "../src/structs/Subnet.sol";
import "../src/lib/SubnetIDHelper.sol";
import "../src/lib/CheckpointHelper.sol";

contract SubnetActorTest is Test {

    using SubnetIDHelper for SubnetID;
    using CheckpointHelper for BottomUpCheckpoint;

    SubnetActor sa;
    Gateway gw;

    address private constant DEFAULT_IPC_GATEWAY_ADDR = address(1024);
    uint64 constant DEFAULT_CHECKPOINT_PERIOD = 10;
    string private constant DEFAULT_NETWORK_NAME = "test";
    uint256 private constant DEFAULT_MIN_VALIDATOR_STAKE = 1 ether;
    uint64 private constant DEFAULT_MIN_VALIDATORS = 1;
    bytes private constant GENESIS = EMPTY_BYTES;
    uint256 constant CROSS_MSG_FEE = 10 gwei;
    uint8 private constant DEFAULT_MAJORITY_PERCENTAGE = 70;
    address GATEWAY_ADDRESS;

    function setUp() public
    {
        address[] memory path = new address[](1);
        // root
        path[0] = address(0);

        Gateway.ConstructorParams memory constructorParams = Gateway.ConstructorParams({
            networkName: SubnetID({route: path}),
            bottomUpCheckPeriod: DEFAULT_CHECKPOINT_PERIOD,
            topDownCheckPeriod: DEFAULT_CHECKPOINT_PERIOD,
            msgFee: CROSS_MSG_FEE,
            majorityPercentage: DEFAULT_MAJORITY_PERCENTAGE
        });
        gw = new Gateway(constructorParams);

        GATEWAY_ADDRESS = address(gw);

        _assertDeploySubnetActor(DEFAULT_NETWORK_NAME, GATEWAY_ADDRESS, ConsensusType.Dummy, DEFAULT_MIN_VALIDATOR_STAKE, DEFAULT_MIN_VALIDATORS, DEFAULT_CHECKPOINT_PERIOD, GENESIS, DEFAULT_MAJORITY_PERCENTAGE);        
    }

    function testDeployment(string calldata _networkName, address _ipcGatewayAddr, uint256 _minActivationCollateral, uint64 _minValidators, uint64 _checkPeriod, bytes calldata _genesis, uint8 _majorityPercentage) public {
        vm.assume(_minActivationCollateral > DEFAULT_MIN_VALIDATOR_STAKE);
        vm.assume(_checkPeriod > DEFAULT_CHECKPOINT_PERIOD);
        vm.assume(_majorityPercentage <= 100);

        _assertDeploySubnetActor(_networkName, _ipcGatewayAddr, ConsensusType.Dummy, _minActivationCollateral, _minValidators, _checkPeriod, _genesis, _majorityPercentage);
        
        SubnetID memory parent = sa.getParent();
        require(parent.isRoot(), "parent.isRoot()");
    }

    function test_Join_Fail_NoMinColalteral() public {
        address validator = vm.addr(100);

        vm.deal(validator, 1 gwei);
        vm.prank(validator);
        vm.expectRevert("a minimum collateral is required to join the subnet");

        sa.join();
    }

    function test_Join_Fail_AlreadyKilled() public {
        address validator = vm.addr(1235);

        _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE);
        _assertLeave(validator, DEFAULT_MIN_VALIDATOR_STAKE);
        _assertKill(validator);

        vm.expectRevert("the subnet is already in a killed or terminating state");
        vm.prank(validator);
        vm.deal(validator, DEFAULT_MIN_VALIDATOR_STAKE + 1);

        sa.join{value: DEFAULT_MIN_VALIDATOR_STAKE}();
    }

    function test_Join_Works_CallAddStake() public {
        address validator = vm.addr(1235);

        _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE);

        vm.expectCall(GATEWAY_ADDRESS, DEFAULT_MIN_VALIDATOR_STAKE, abi.encodeWithSelector(gw.addStake.selector), 1);

        _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE);
        
        require(sa.validatorCount() == 1);
        require(sa.validatorAt(0) == validator);
    }

    function test_Join_Works_CallRegister() public {
        address validator = vm.addr(1235);

        vm.expectCall(GATEWAY_ADDRESS, DEFAULT_MIN_VALIDATOR_STAKE, abi.encodeWithSelector(gw.register.selector), 1);
        
        _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE);
    }

    function test_Join_Works_MultipleNewValidators() public {
        _assertJoin(vm.addr(1234), DEFAULT_MIN_VALIDATOR_STAKE);
        _assertJoin(vm.addr(1235), DEFAULT_MIN_VALIDATOR_STAKE);

        require(sa.validatorCount() == 2);
    }

    function test_Join_Works_NoNewValidator_CollateralNotEnough() public {
        address validator = vm.addr(1235);
        
         _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE - 1);

        require(sa.validatorCount() == 0);
    }

    function test_Join_Works_DelegatedConsensusType() public {
        _assertDeploySubnetActor(DEFAULT_NETWORK_NAME, GATEWAY_ADDRESS, ConsensusType.Delegated, DEFAULT_MIN_VALIDATOR_STAKE, DEFAULT_MIN_VALIDATORS, DEFAULT_CHECKPOINT_PERIOD, GENESIS, DEFAULT_MAJORITY_PERCENTAGE);        
    
        _assertJoin(vm.addr(1234), DEFAULT_MIN_VALIDATOR_STAKE);
        _assertJoin(vm.addr(1235), DEFAULT_MIN_VALIDATOR_STAKE);
        
        require(sa.validatorCount() == 1);
    }

    function test_Join_Works_ReactivateSubnet() public {
        _assertJoin(vm.addr(1234), DEFAULT_MIN_VALIDATOR_STAKE);
        _assertLeave(vm.addr(1234), DEFAULT_MIN_VALIDATOR_STAKE);

        require(sa.totalStake() == 0);
        require(sa.validatorCount() == 0);
        require(sa.status() == Status.Inactive);

        _assertJoin(vm.addr(1235), DEFAULT_MIN_VALIDATOR_STAKE);
        
        require(sa.validatorCount() == 1);
    }

    function test_Leave_Works_NoValidatorsLeft() public payable {
        address validator = address(1235);
        uint256 amount = DEFAULT_MIN_VALIDATOR_STAKE;

        _assertJoin(validator, amount);

        _assertLeave(validator, amount);

        require(sa.totalStake() == 0);
        require(sa.validatorCount() == 0);
        require(sa.status() == Status.Inactive);
    }

    function test_Leave_Fail_AlreadyKilled() public payable {  
        address validator = address(1235);
        uint256 amount = DEFAULT_MIN_VALIDATOR_STAKE;

        _assertJoin(validator, amount);

        _assertLeave(validator, amount);
        _assertKill(validator);

        vm.prank(validator);
        vm.deal(validator, amount);
        vm.expectRevert("the subnet is already in a killed or terminating state");

        sa.leave();
    }

    function test_Leave_Fail_NoStake() public payable {  
        address caller = address(1235);

        vm.prank(caller);
        vm.deal(caller, 1 ether);
        vm.expectRevert("caller has no stake in subnet");

        sa.leave();
    }

    function test_Kill_Works() public payable {
        address validator = address(1235);

        _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE);
        _assertLeave(validator, DEFAULT_MIN_VALIDATOR_STAKE);

        _assertKill(validator);

        require(GATEWAY_ADDRESS.balance == 0);
    }

    function test_Kill_Fails_NotAllValidatorsLeft() public payable {
        address validator1 = address(1235);
        address validator2 = address(1236);
       
        _assertJoin(validator1, DEFAULT_MIN_VALIDATOR_STAKE);
        _assertJoin(validator2, DEFAULT_MIN_VALIDATOR_STAKE);

        _assertLeave(validator1, DEFAULT_MIN_VALIDATOR_STAKE);

        vm.prank(validator1);
        vm.expectRevert("this subnet can only be killed when all validators have left");
        sa.kill();
    }

    function test_Kill_Fails_AlreadyTerminating() public {
        address validator = vm.addr(1235);
        
        _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE);
        _assertLeave(validator, DEFAULT_MIN_VALIDATOR_STAKE);

        _assertKill(validator);

        vm.prank(validator);
        vm.expectRevert("the subnet is already in a killed or terminating state");
        sa.kill();
    }

    function test_Kill_Fails_CollateralNotZero() public {
        address validator = vm.addr(1235);
        
        _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE);
        _assertLeave(validator, DEFAULT_MIN_VALIDATOR_STAKE);

        vm.prank(GATEWAY_ADDRESS);
        vm.deal(GATEWAY_ADDRESS, 1 ether);

        (bool success, ) = address(sa).call{value: 1}("");
        
        require(success, "funding SubnetActor failed");

        vm.prank(validator);
        vm.expectRevert("there is still collateral in the subnet");
        sa.kill();
    }

    function test_SubmitCheckpoint_Works() public {
        address validator = vm.addr(100);
        _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE);
        address validator2 = vm.addr(101);
        _assertJoin(validator2, DEFAULT_MIN_VALIDATOR_STAKE);
        address validator3 = vm.addr(102);
        _assertJoin(validator3, DEFAULT_MIN_VALIDATOR_STAKE);

        BottomUpCheckpoint memory checkpoint = _createBottomUpCheckpoint();

        vm.prank(validator);
        sa.submitCheckpoint(checkpoint);

        require(sa.hasValidatorVotedForSubmission(checkpoint.epoch, validator) == true);

        vm.prank(validator2);
        sa.submitCheckpoint(checkpoint);

        require(sa.hasValidatorVotedForSubmission(checkpoint.epoch, validator2) == true);

        vm.prank(validator3);
        vm.expectCall(address(this), abi.encodeWithSelector(this.callback.selector));
        sa.submitCheckpoint(checkpoint);

        require(sa.hasValidatorVotedForSubmission(checkpoint.epoch, validator3) == true);
    }

    function callback() public view {
        console.log("callback called");
    }

    function test_SubmitCheckpoint_AddsVoter() public  {
        address validator = vm.addr(100);
        _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE);
        address validator2 = vm.addr(101);
        _assertJoin(validator2, DEFAULT_MIN_VALIDATOR_STAKE);
        
        BottomUpCheckpoint memory checkpoint = _createBottomUpCheckpoint();

        vm.prank(validator);
        sa.submitCheckpoint(checkpoint);

        require(sa.hasValidatorVotedForSubmission(checkpoint.epoch, validator) == true);
    }

    function test_SubmitCheckpoint_Fails_InvalidValidator() public {
        address validator = vm.addr(100);
        _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE);
        
        BottomUpCheckpoint memory checkpoint = _createBottomUpCheckpoint();

        address notValidator = vm.addr(200);
        vm.prank(notValidator);
        vm.deal(notValidator, 1);
        vm.expectRevert("not validator");
        sa.submitCheckpoint(checkpoint);
    }

    function test_SubmitCheckpoint_Fails_CheckpointAlreadyCommited() public {
        address validator = vm.addr(100);
        _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE);

        BottomUpCheckpoint memory checkpoint = _createBottomUpCheckpoint();

        vm.prank(validator);
        sa.submitCheckpoint(checkpoint);

        checkpoint.epoch = 1;

        vm.prank(validator);
        vm.expectRevert("epoch already executed");
        sa.submitCheckpoint(checkpoint);
    }

    function test_SubmitCheckpoint_Fails_ValidatorAlreadyVoted() public {
        address validator = vm.addr(100);
        _assertJoin(validator, DEFAULT_MIN_VALIDATOR_STAKE);

        address validator2 = vm.addr(200);
        _assertJoin(validator2, DEFAULT_MIN_VALIDATOR_STAKE);

        BottomUpCheckpoint memory checkpoint = _createBottomUpCheckpoint();

        vm.startPrank(validator);
        sa.submitCheckpoint(checkpoint);

        vm.expectRevert("validator has already voted");
        sa.submitCheckpoint(checkpoint);
    }

    function _assertJoin(address validator, uint256 amount) internal { 
        vm.startPrank(validator);
        vm.deal(validator, amount + 1);

        uint256 balanceBefore = validator.balance;
        uint256 stakeBefore = sa.stake(validator);
        uint256 totalStakeBefore = sa.totalStake();

        sa.join{value: amount}();

        require(sa.stake(validator) == stakeBefore + amount);
        require(sa.totalStake() == totalStakeBefore + amount);
        require(validator.balance == balanceBefore - amount);

        vm.stopPrank();
    }

    function _assertLeave(address validator, uint256 amount) internal {
        uint256 validatorBalanceBefore = validator.balance;
        uint256 validatorsCountBefore = sa.validatorCount();
        uint256 totalStakeBefore = sa.totalStake();

        vm.prank(validator);
        vm.expectCall(GATEWAY_ADDRESS, abi.encodeWithSelector(gw.releaseStake.selector, amount));
        vm.expectCall(validator, amount, EMPTY_BYTES);

        sa.leave();

        require(sa.stake(validator) == 0);
        require(sa.totalStake() == totalStakeBefore - amount);
        require(sa.validatorCount() == validatorsCountBefore - 1);
        require(validator.balance == validatorBalanceBefore + amount);
    }

    function _assertKill(address validator) internal {
        vm.startPrank(validator);
        vm.deal(validator, 1 ether);
        vm.expectCall(GATEWAY_ADDRESS, abi.encodeWithSelector(gw.kill.selector));

        sa.kill();

        require(sa.totalStake() == 0);
        require(sa.validatorCount() == 0);
        require(sa.status() == Status.Killed);

        vm.stopPrank();
    }

    function _assertDeploySubnetActor(string memory _name, address _ipcGatewayAddr, ConsensusType _consensus, uint256 _minActivationCollateral, uint64 _minValidators, uint64 _checkPeriod, bytes memory _genesis, uint8 _majorityPercentage) public {
        SubnetID memory _parentId = gw.getNetworkName();

        sa = new SubnetActor(SubnetActor.ConstructParams({
            parentId: _parentId,
            name: _name,
            ipcGatewayAddr: _ipcGatewayAddr,
            consensus: _consensus,
            minActivationCollateral: _minActivationCollateral,
            minValidators: _minValidators,
            bottomUpCheckPeriod: _checkPeriod,
            topDownCheckPeriod: _checkPeriod,
            majorityPercentage: _majorityPercentage,
            genesis: _genesis
        }));

        require(keccak256(abi.encodePacked(sa.name())) == keccak256(abi.encodePacked(_name)), "keccak256(abi.encodePacked(sa.name())) == keccak256(abi.encodePacked(_networkName))");
        require(sa.ipcGatewayAddr() == _ipcGatewayAddr, "sa.ipcGatewayAddr() == _ipcGatewayAddr");
        require(sa.minActivationCollateral() == _minActivationCollateral, "sa.minActivationCollateral() == _minActivationCollateral");
        require(sa.minValidators() == _minValidators, "sa.minValidators() == _minValidators");
        require(sa.topDownCheckPeriod() == _checkPeriod, "sa.topDownCheckPeriod() == _checkPeriod");
        require(keccak256(sa.genesis()) == keccak256(_genesis), "keccak256(sa.genesis()) == keccak256(_genesis)");
        require(sa.majorityPercentage() == _majorityPercentage, "sa.majorityPercentage() == _majorityPercentage");
        require(sa.consensus() == _consensus);
        require(sa.getParent().toHash() == _parentId.toHash(), "parent.toHash() == SubnetID({route: path}).toHash()");
    }

    function _createBottomUpCheckpoint() internal view returns (BottomUpCheckpoint memory checkpoint){
        address[] memory route = new address[](2);
        route[0] = address(0);
        route[1] = address(sa);
        SubnetID memory source = SubnetID({route: route});
        CrossMsg[] memory crossMsgs = new CrossMsg[](1);
        crossMsgs[0] = CrossMsg({
                message: StorableMsg({
                    from: IPCAddress({
                        subnetId: SubnetID({route: new address[](0)}),
                        rawAddress: address(this)
                    }),
                    to: IPCAddress({
                        subnetId: gw.getNetworkName(),
                        rawAddress: address(this)
                    }),
                    value: 0,
                    nonce: 0,
                    method: this.callback.selector,
                    params: new bytes(0)
                }),
                wrapped: false
            });
        checkpoint = BottomUpCheckpoint({source: source, epoch: DEFAULT_CHECKPOINT_PERIOD, fee: 0, crossMsgs: crossMsgs, prevHash: EMPTY_HASH, children: new ChildCheck[](0)});

    }

}