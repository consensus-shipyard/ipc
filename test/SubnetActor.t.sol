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
    using CheckpointHelper for Checkpoint;

    SubnetActor sa;
    Gateway gw;

    address private constant DEFAULT_IPC_GATEWAY_ADDR = address(1024);
    int64 constant DEFAULT_CHECKPOINT_PERIOD = 10;
    string private constant DEFAULT_NETWORK_NAME = "test";
    uint256 private constant DEFAULT_MIN_VALIDATOR_STAKE = 1 ether;
    uint64 private constant DEFAULT_MIN_VALIDATORS = 1;
    int64 private constant DEFAULT_FINALITY_TRESHOLD = 1;
    int64 private constant DEFAULT_CHECK_PERIOD = 50;
    bytes private constant GENESIS = EMPTY_BYTES;
    uint256 constant CROSS_MSG_FEE = 10 gwei;
    uint8 private constant DEFAULT_MAJORITY_PERCENTAGE = 70;

    function setUp() public
    {
        address[] memory path = new address[](1);
        path[0] = address(0);
        gw = new Gateway(path, DEFAULT_CHECKPOINT_PERIOD, CROSS_MSG_FEE);

        path[0] = address(gw);
        SubnetID memory parentId = SubnetID(path);
        sa = new SubnetActor(parentId, DEFAULT_NETWORK_NAME, address(gw), ConsensusType.Dummy, DEFAULT_MIN_VALIDATOR_STAKE, DEFAULT_MIN_VALIDATORS, DEFAULT_FINALITY_TRESHOLD, DEFAULT_CHECK_PERIOD, GENESIS, DEFAULT_MAJORITY_PERCENTAGE);
    
    }

    function testDeployment(address _ipcGatewayAddr, uint256 _minValidatorStake, uint64 _minValidators, int64 _finalityTreshold, int64 _checkPeriod, bytes calldata _genesis, uint8 _majorityPercentage) public {
        vm.assume(_minValidatorStake > 0);
        vm.assume(_minValidators > 0);
        vm.assume(_majorityPercentage <= 100);

        address[] memory path = new address[](1);
        path[0] = address(_ipcGatewayAddr);
        SubnetID memory parentId = SubnetID(path);
        sa = new SubnetActor(parentId, DEFAULT_NETWORK_NAME, _ipcGatewayAddr, ConsensusType.Dummy, _minValidatorStake, _minValidators, _finalityTreshold, _checkPeriod, _genesis, _majorityPercentage);
    
        require(keccak256(abi.encodePacked(sa.name())) == keccak256(abi.encodePacked(DEFAULT_NETWORK_NAME)));
        require(sa.ipcGatewayAddr() == _ipcGatewayAddr);
        require(sa.consensus() == ConsensusType.Dummy);
        require(sa.minValidatorStake() == _minValidatorStake);
        require(sa.minValidators() == _minValidators);
        require(sa.finalityThreshold() == _finalityTreshold);
        require(sa.checkPeriod() == _checkPeriod);
        require(keccak256(sa.genesis()) == keccak256(_genesis));
        require(sa.majorityPercentage() == _majorityPercentage);

        SubnetID memory subnet = sa.getParent();
        require(subnet.isRoot());
        require(subnet.toHash() == parentId.toHash());
    }

    function test_Join_Fail_NoMinColalteral() public payable {
        address validator = vm.addr(100);
        vm.prank(validator);
        vm.expectRevert("a minimum collateral is required to join the subnet");
        sa.join();
    }

    function test_Join_Works(uint256 amount) public payable {
        vm.assume(amount > 1 ether);

        address validator = vm.addr(1235);

        vm.prank(validator);
        vm.deal(validator, amount);
        (bool success, ) = address(sa).call{value: amount}(abi.encodeWithSignature("join()"));
        require(success);
        
        require(sa.stake(validator) == amount);
        require(sa.totalStake() == amount);
        require(sa.validatorCount() == 1);
        require(sa.validatorAt(0) == validator);
    }

    function test_Join_CallRegister() public {
        address validator = vm.addr(1235);

        vm.prank(validator);
        vm.deal(validator, DEFAULT_MIN_VALIDATOR_STAKE);
        vm.expectCall(address(gw), DEFAULT_MIN_VALIDATOR_STAKE, abi.encodeWithSignature("register()"));
        (bool success, ) = address(sa).call{value: DEFAULT_MIN_VALIDATOR_STAKE}(abi.encodeWithSignature("join()"));
        require(success);
    }

    function test_Join_CallAddStake_SubnetAlreadyActive() public {
        address validator = vm.addr(1235);

        _join(validator);

        vm.prank(validator);
        vm.deal(validator, DEFAULT_MIN_VALIDATOR_STAKE / 2);
        vm.expectCall(address(gw), DEFAULT_MIN_VALIDATOR_STAKE / 2, abi.encodeWithSignature("addStake()"));
        (bool success, ) = address(sa).call{value: DEFAULT_MIN_VALIDATOR_STAKE / 2}(abi.encodeWithSignature("join()"));
        require(success);
    }

    function test_Join_NoNewValidator_ValueLowerThanMinStake() public {
        address validator = vm.addr(1235);
        vm.prank(validator);
        vm.deal(validator, DEFAULT_MIN_VALIDATOR_STAKE - 1);
        (bool success, ) = address(sa).call{value: DEFAULT_MIN_VALIDATOR_STAKE - 1}(abi.encodeWithSignature("join()"));
        require(success);

        require(sa.stake(validator) == DEFAULT_MIN_VALIDATOR_STAKE - 1);
        require(sa.totalStake() == DEFAULT_MIN_VALIDATOR_STAKE - 1);
        require(sa.validatorCount() == 0);
    }

    function test_Join_NoNewValidator_AlreadyExists() public {
        address validator = vm.addr(1235);

        _join(validator);
        _join(validator);
        require(sa.stake(validator) == 2 * DEFAULT_MIN_VALIDATOR_STAKE);
        require(sa.validatorCount() == 1);
    }

    function test_Join_NoNewValidator_DelegatedConsensusType_ValidatorAlreadyJoined() public {
        address[] memory path = new address[](1);
        path[0] = address(gw);
        SubnetID memory parentId = SubnetID(path);
        sa = new SubnetActor(parentId, DEFAULT_NETWORK_NAME, address(gw), ConsensusType.Delegated, DEFAULT_MIN_VALIDATOR_STAKE, DEFAULT_MIN_VALIDATORS, DEFAULT_FINALITY_TRESHOLD, DEFAULT_CHECK_PERIOD, GENESIS, DEFAULT_MAJORITY_PERCENTAGE);
    
        address validator = vm.addr(1235);
        _join(validator);

        address validator2 = vm.addr(1236);
        _join(validator2);
        require(sa.validatorCount() == 1);
        require(sa.stake(validator) == DEFAULT_MIN_VALIDATOR_STAKE);
        require(sa.stake(validator2) == DEFAULT_MIN_VALIDATOR_STAKE);
    }

    function test_Leave_Works() public payable {
        address validator = address(1235);
        _join(validator);

        vm.prank(validator);
        vm.expectCall(address(gw), abi.encodeWithSignature("releaseStake(uint256)", DEFAULT_MIN_VALIDATOR_STAKE));
        vm.expectCall(validator, DEFAULT_MIN_VALIDATOR_STAKE, bytes(""));
        sa.leave();

        require(sa.stake(validator) == 0);
        require(sa.totalStake() == 0);
        require(sa.validatorCount() == 0);
    }

    function test_Leave_Fail_NoStake() public payable {
        address validator = address(1235);
        vm.prank(validator);
        vm.expectRevert();
        sa.leave();
    }

    function test_Kill_Works() public payable {

        address validator = address(1235);
        _join(validator);

        vm.startPrank(validator);
        sa.leave();

        vm.expectCall(address(gw), abi.encodeWithSignature("kill()"));
        sa.kill();

        require(address(gw).balance == 0);
        require(sa.totalStake() == 0);
        require(sa.validatorCount() == 0);
    }

    function test_Kill_Fails_NotAllValidatorsLeft() public payable {

        address validator1 = address(1235);
        address validator2 = address(1236);
       
        _join(validator1);
        _join(validator2);

        vm.prank(validator1);
        sa.leave();

        vm.prank(validator1);
        vm.expectRevert("this subnet can only be killed when all validators have left");
        sa.kill();
    }

    function test_Kill_Fails_AlreadyTerminating() public {
        address validator = vm.addr(1235);
        _join(validator);
        vm.startPrank(validator);
        sa.leave();
        sa.kill();
        vm.expectRevert("the subnet is already in a killed or terminating state");
        sa.kill();
    }

    function test_SubmitCheckpoint_Works() public {
        address validator = vm.addr(100);
        _join(validator);
        address validator2 = vm.addr(101);
        _join(validator2);
        address validator3 = vm.addr(102);
        _join(validator3);

        CheckData memory data = _createCheckData(100);
        Checkpoint memory checkpoint = Checkpoint({data: data, signature: bytes("")});
        bytes32 checkpointHash = checkpoint.toHash();
        
        Checkpoint memory checkpoint1 = signCheckpoint(100, data);

        vm.prank(validator);
        sa.submitCheckpoint(checkpoint1);


        require(sa.windowCheckCount(checkpointHash) == 1);
        require(sa.windowCheckAt(checkpointHash, 0) == validator);

        Checkpoint memory checkpoint2 = signCheckpoint(101, data);

        vm.prank(validator2);
        sa.submitCheckpoint(checkpoint2);

        require(sa.windowCheckCount(checkpointHash) == 2);
        require(sa.windowCheckAt(checkpointHash, 1) == validator2);

        Checkpoint memory checkpoint3 = signCheckpoint(102, data);

        vm.prank(validator3);
        vm.expectCall(address(gw), abi.encodeWithSelector(gw.commitChildCheck.selector, checkpoint3));
        sa.submitCheckpoint(checkpoint3);

        require(sa.windowCheckCount(checkpointHash) == 0);
    }

    function signCheckpoint(uint pk, CheckData memory data) internal pure returns(Checkpoint memory) {
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(pk, keccak256(abi.encode(data)));
        return Checkpoint({data: data, signature: abi.encodePacked(r, s, v)});
    }

    function test_SubmitCheckpoint_AddsVoter() public  {
        address validator = vm.addr(100);
        _join(validator);
        address validator2 = vm.addr(101);
        _join(validator2);

        CheckData memory data = _createCheckData(100);
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(100, keccak256(abi.encode(data)));
        
        Checkpoint memory checkpoint = Checkpoint({data: data, signature: abi.encodePacked(r, s, v)});

        vm.prank(validator);
        sa.submitCheckpoint(checkpoint);

        bytes32 checkpointHash = checkpoint.toHash();
        require(sa.windowCheckCount(checkpointHash) == 1);
        require(sa.windowCheckAt(checkpointHash, 0) == validator);
    }


    function test_SubmitCheckpoint_Fails_InvalidSignture() public {
        address validator = vm.addr(100);
        _join(validator);

        CheckData memory data = _createCheckData(100);
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(200, keccak256(abi.encode(data)));
        
        Checkpoint memory checkpoint = Checkpoint({data: data, signature: abi.encodePacked(r, s, v)});

        vm.prank(validator);
        vm.expectRevert("invalid signature");
        sa.submitCheckpoint(checkpoint);
    }

    function test_SubmitCheckpoint_Fails_InvalidValidator() public {
        address validator = vm.addr(100);
        _join(validator);

        CheckData memory data = _createCheckData(100);
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(100, keccak256(abi.encode(data)));
        
        Checkpoint memory checkpoint = Checkpoint({data: data, signature: abi.encodePacked(r, s, v)});

        vm.prank(vm.addr(200));
        vm.expectRevert("not validator");
        sa.submitCheckpoint(checkpoint);
    }

    function test_SubmitCheckpoint_Fails_SubnetInactive() public {
        address validator = vm.addr(100);
                
        vm.startPrank(validator);
        vm.deal(validator, DEFAULT_MIN_VALIDATOR_STAKE / 2);
        (bool success, ) = address(sa).call{value: DEFAULT_MIN_VALIDATOR_STAKE / 2}(abi.encodeWithSignature("join()"));
        require(success);

        CheckData memory data = _createCheckData(100); 
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(100, keccak256(abi.encode(data)));
        
        Checkpoint memory checkpoint = Checkpoint({data: data, signature: abi.encodePacked(r, s, v)});

        vm.expectRevert("not validator");
        sa.submitCheckpoint(checkpoint);
    }

    function test_SubmitCheckpoint_Fails_CheckpointAlreadyCommited() public {
        address validator = vm.addr(100);
        _join(validator);

        CheckData memory data = _createCheckData(100); 

        (uint8 v, bytes32 r, bytes32 s) = vm.sign(100, keccak256(abi.encode(data)));
        Checkpoint memory checkpoint = Checkpoint({data: data, signature: abi.encodePacked(r, s, v)});
        vm.startPrank(validator);
        sa.submitCheckpoint(checkpoint);

        vm.expectRevert("cannot submit checkpoint for epoch");
        sa.submitCheckpoint(checkpoint);
    }

    function test_SubmitCheckpoint_Fails_OutsideOfSigningWindow() public {
        address validator = vm.addr(100);
        _join(validator);
        address validator2 = vm.addr(101);
        _join(validator2);
        address validator3 = vm.addr(102);
        _join(validator3);

        CheckData memory data = _createCheckData(125);
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(100, keccak256(abi.encode(data)));
        
        Checkpoint memory checkpoint = Checkpoint({data: data, signature: abi.encodePacked(r, s, v)});

        vm.prank(validator);
        vm.expectRevert("epoch in checkpoint doesn't correspond with a signing window");
        sa.submitCheckpoint(checkpoint);
    }

    function test_SubmitCheckpoint_Fails_ValidatorAlreadyVoted() public {
        address validator = vm.addr(100);
        _join(validator);

        address validator2 = vm.addr(200);
        _join(validator2);


        CheckData memory data = _createCheckData(100); 
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(100, keccak256(abi.encode(data)));
        
        Checkpoint memory checkpoint = Checkpoint({data: data, signature: abi.encodePacked(r, s, v)});

        vm.startPrank(validator);
        sa.submitCheckpoint(checkpoint);

        vm.expectRevert("miner has already voted the checkpoint");
        sa.submitCheckpoint(checkpoint);
    }


    function _createCheckData(int64 epoch) internal view returns (CheckData memory data){
        SubnetID memory subnet = sa.getParent().createSubnetId(address(sa));

        CrossMsgMeta memory crossMsgMeta = CrossMsgMeta({msgsHash: EMPTY_HASH, value: 0, nonce: 0, fee: 0});

        ChildCheck[] memory children = new ChildCheck[](1);
        bytes32[] memory checks = new bytes32[](0);
        children[0] = ChildCheck({source: subnet, checks: checks});

        data = CheckData({source: subnet, tipSet: EMPTY_BYTES, epoch: epoch, prevHash: CheckpointHelper.EMPTY_CHECKPOINT_DATA_HASH, children: children, crossMsgs: crossMsgMeta });
    }

    function _join(address _validator) internal {
        uint256 amount = DEFAULT_MIN_VALIDATOR_STAKE;

        vm.prank(_validator);
        vm.deal(_validator, amount);
        (bool success, ) = address(sa).call{value: amount}(abi.encodeWithSignature("join()"));
        require(success);
    }

}