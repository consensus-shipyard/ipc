// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.7;

import "./structs/Checkpoint.sol";
import "./structs/Postbox.sol";
import "./enums/Status.sol";
import "./interfaces/IGateway.sol";
import "./interfaces/ISubnetActor.sol";
import "./lib/SubnetIDHelper.sol";
import "./lib/CheckpointMappingHelper.sol";
import "./lib/CheckpointHelper.sol";
import "./lib/AccountHelper.sol";
import "./lib/CrossMsgHelper.sol";
import "openzeppelin-contracts/security/ReentrancyGuard.sol";
import "openzeppelin-contracts/utils/Address.sol";
import "fevmate/utils/FilAddress.sol";

/// @title Gateway Contract
/// @author LimeChain team
contract Gateway is IGateway, ReentrancyGuard {
    using FilAddress for address;
    using FilAddress for address payable;
    using AccountHelper for address;
    using SubnetIDHelper for SubnetID;
    using CheckpointHelper for Checkpoint;
    using CheckpointMappingHelper for mapping(int64 => Checkpoint);

    int64 constant DEFAULT_CHECKPOINT_PERIOD = 10;
    uint64 constant MIN_COLLATERAL_AMOUNT = 1 ether;
    uint64 constant MAX_NONCE = type(uint64).max;

    /// @notice ID of the current network
    SubnetID private networkName;

    /// @notice Number of active subnets spawned from this one
    uint64 public totalSubnets;

    /// @notice Minimum stake required to create a new subnet
    uint256 public minStake;

    /// @notice List of subnets
    /// SubnetID => Subnet
    mapping(bytes32 => Subnet) public subnets;

    /// @notice Checkpoint period in number of epochs for the subnet
    int64 public checkPeriod;

    /// @notice Checkpoint templates in the GW per epoch
    mapping(int64 => Checkpoint) public checkpoints;

    /// @notice Stores information about the list of messages and child msgMetas being propagated in checkpoints to the top of the hierarchy.
    mapping(int64 => CrossMsg[]) public crossMsgRegistry;

    /// @notice Indicates if a crossMsg already exists for a given epoch(checkpoint)
    mapping(int64 => mapping(bytes32 => bool)) public crossMsgExistInRegistry;

    /// @notice Stores a pointer to CrossMsg[](crossMsgRegistry) for the given epoch
    mapping(int64 => bytes32) public crossMsgCidRegistry;

    /// @notice Stores an epoch for the given pointer to CrossMsg[]
    mapping(bytes32 => int64) public crossMsgEpochRegistry;

    uint256 public lastPostboxId;
    /// @notice Postbox keeps track for an EOA of all the cross-net messages triggered by
    /// an actor that need to be propagated further through the hierarchy.
    /// postbox id => PostBoxItem
    mapping(uint64 => PostBoxItem) private postbox;

    /// @notice Latest nonce of a cross message sent from subnet.
    uint64 public nonce;

    /// @notice Nonce of bottom-up messages for msgMeta received from checkpoints.
    /// This nonce is used to mark with a nonce the metadata about cross-net
    /// messages received in checkpoints. This is used to order the
    /// bottom-up cross-net messages received through checkpoints.
    uint64 public bottomUpNonce;

    /// @notice Queue of bottom-up cross-net messages to be applied.
    /// bottom up nonce => CrossMsgMeta
    mapping(uint64 => CrossMsgMeta) public bottomUpMsgMeta;

    /// @notice AppliedNonces keep track of the next nonce of the message to be applied.
    /// This prevents potential replay attacks.
    uint64 public appliedBottomUpNonce;
    uint64 public appliedTopDownNonce;

    /// @notice fee amount charged per cross message
    uint256 public crossMsgFee;

    /// epoch => SubnetID => [childIndex, exists(0 - no, 1 - yes)]
    mapping(int64 => mapping(bytes32 => uint256[2])) internal children;
    /// epoch => SubnetID => check => exists
    mapping(int64 => mapping(bytes32 => mapping(bytes32 => bool)))
        internal checks;

    modifier signableOnly() {
        require(
            msg.sender.isAccount() || msg.sender.isMultisig(),
            "the caller is not an account nor a multi-sig"
        );

        _;
    }

    constructor(address[] memory path, int64 checkpointPeriod, uint256 msgFee) {
        networkName = SubnetID(path);
        minStake = MIN_COLLATERAL_AMOUNT;
        checkPeriod = checkpointPeriod > DEFAULT_CHECKPOINT_PERIOD
            ? checkpointPeriod
            : DEFAULT_CHECKPOINT_PERIOD;
        appliedBottomUpNonce = MAX_NONCE;
        crossMsgFee = msgFee;
    }

    function getCrossMsgsLength(
        bytes32 msgsHash
    ) external view returns (uint256) {
        int64 epoch = crossMsgEpochRegistry[msgsHash];
        return crossMsgRegistry[epoch].length;
    }

    function getCrossMsg(
        bytes32 msgsHash,
        uint256 index
    ) external view returns (CrossMsg memory) {
        int64 epoch = crossMsgEpochRegistry[msgsHash];
        return crossMsgRegistry[epoch][index];
    }

    function getSubnetTopDownMsgsLength(
        SubnetID memory subnetId
    ) public view returns (uint) {
        (, Subnet storage subnet) = getSubnet(subnetId);

        return subnet.topDownMsgs.length;
    }

    function getSubnetTopDownMsg(
        SubnetID memory subnetId,
        uint index
    ) public view returns (CrossMsg memory) {
        (, Subnet storage subnet) = getSubnet(subnetId);
        return subnet.topDownMsgs[index];
    }

    function getNetworkName() external view returns (SubnetID memory) {
        return networkName;
    }

    function register() external payable {
        require(
            msg.value >= minStake,
            "call to register doesn't include enough funds"
        );

        (bool registered, Subnet storage subnet) = getSubnet(msg.sender);

        require(registered == false, "subnet is already registered");

        subnet.id = networkName.createSubnetId(msg.sender);
        subnet.stake = msg.value;
        subnet.status = Status.Active;
        subnet.nonce = 0;
        subnet.circSupply = 0;

        totalSubnets += 1;
    }

    function addStake() external payable {
        require(msg.value > 0, "no stake to add");

        (bool registered, Subnet storage subnet) = getSubnet(msg.sender);

        require(registered, "subnet is not registered");

        subnet.stake += msg.value;
    }

    function releaseStake(uint amount) external nonReentrant {
        require(amount > 0, "no funds to release in params");

        (bool registered, Subnet storage subnet) = getSubnet(msg.sender);

        require(registered, "subnet is not registered");
        require(
            subnet.stake >= amount,
            "subnet actor not allowed to release so many funds"
        );
        require(
            address(this).balance >= amount,
            "something went really wrong! the actor doesn't have enough balance to release"
        );

        subnet.stake -= amount;

        if (subnet.stake < minStake) {
            subnet.status = Status.Inactive;
        }

        payable(subnet.id.getActor()).sendValue(amount);
    }

    function kill() external {
        (bool registered, Subnet storage subnet) = getSubnet(msg.sender);

        require(registered, "subnet is not registered");
        require(
            address(this).balance >= subnet.stake,
            "something went really wrong! the actor doesn't have enough balance to release"
        );
        require(
            subnet.circSupply == 0,
            "cannot kill a subnet that still holds user funds in its circ. supply"
        );

        uint256 stake = subnet.stake;

        totalSubnets -= 1;

        delete subnets[subnet.id.toHash()];

        payable(msg.sender).sendValue(stake);
    }

    function commitChildCheck(
        Checkpoint calldata commit
    ) external returns (uint fee) {
        require(
            commit.data.source.getActor().normalize() == msg.sender,
            "source in checkpoint doesn't belong to subnet"
        );

        (bool registered, Subnet storage subnet) = getSubnet(msg.sender);

        require(registered, "subnet is not registered");

        require(
            subnet.status == Status.Active,
            "can't commit checkpoint for an inactive subnet"
        );

        require(
            subnet.prevCheckpoint.data.epoch <= commit.data.epoch,
            "checkpoint being committed belongs to the past"
        );
        if (commit.data.prevHash != EMPTY_HASH) {
            require(
                subnet.prevCheckpoint.toHash() == commit.data.prevHash,
                "previous checkpoint not consistent with previous one"
            );
        }

        // cross message
        if (commit.hasCrossMsgMeta()) {
            if (commit.data.crossMsgs.msgsHash != EMPTY_HASH) {
                bottomUpMsgMeta[bottomUpNonce] = commit.data.crossMsgs;
                bottomUpMsgMeta[bottomUpNonce].nonce = bottomUpNonce;
                bottomUpNonce += 1;
            }

            require(
                subnet.circSupply >=
                    commit.data.crossMsgs.value + commit.data.crossMsgs.fee,
                "wtf! we can't release funds below circ, supply. something went really wrong"
            );

            subnet.circSupply -= commit.data.crossMsgs.value;
            fee = commit.data.crossMsgs.fee;
        }

        (
            bool checkpointExists,
            int64 currentEpoch,
            Checkpoint storage checkpoint
        ) = checkpoints.getCheckpointPerEpoch(block.number, checkPeriod);

        // create checkpoint if not exists
        if (checkpointExists == false) {
            checkpoint.data.source = networkName;
            checkpoint.data.epoch = currentEpoch;
        }

        bytes32 commitSource = commit.data.source.toHash();
        bytes32 commitData = commit.toHash();

        uint[2] memory child = children[currentEpoch][commitSource];
        uint childIndex = child[0]; // index at checkpoint.data.children for the given subnet
        bool childExists = child[1] == 1; // 0 - no, 1 - yes
        bool childCheckExists = checks[currentEpoch][commitSource][commitData];

        require(
            childCheckExists == false,
            "child checkpoint being committed already exists"
        );

        if (childExists == false) {
            checkpoint.data.children.push(
                ChildCheck({
                    source: commit.data.source,
                    checks: new bytes32[](0)
                })
            );
            childIndex = checkpoint.data.children.length - 1;
        }

        checkpoint.data.children[childIndex].checks.push(commitData);

        children[currentEpoch][commitSource][0] = childIndex;
        children[currentEpoch][commitSource][1] = 1;
        checks[currentEpoch][commitSource][commitData] = true;

        subnet.prevCheckpoint = commit;

        if (fee > 0) {
            distributeRewards(msg.sender, fee);
        }
    }

    function fund(SubnetID calldata subnetId) external payable signableOnly {
        require(msg.value > crossMsgFee, "not enough gas to pay cross-message");

        CrossMsg memory crossMsg = CrossMsgHelper.createFundMsg(
            subnetId,
            msg.sender,
            msg.value - crossMsgFee
        );

        // commit top-down message.
        (bool exist, Subnet storage subnet) = getSubnet(
            crossMsg.message.to.subnetId.down(networkName)
        );

        require(exist, "couldn't compute the next subnet in route");

        crossMsg.message.nonce = subnet.nonce;
        subnet.nonce += 1;
        subnet.circSupply += crossMsg.message.value;
        subnet.topDownMsgs.push(crossMsg);

        distributeRewards(subnetId.getActor(), crossMsgFee);
    }

    function release() external payable signableOnly {
        require(msg.value > crossMsgFee, "not enough gas to pay cross-message");

        (, int64 epoch, Checkpoint storage checkpoint) = checkpoints
            .getCheckpointPerEpoch(block.timestamp, checkPeriod);

        uint256 releaseAmount = msg.value - crossMsgFee;

        CrossMsg memory crossMsg = CrossMsgHelper.createReleaseMsg(
            networkName,
            msg.sender,
            releaseAmount,
            nonce
        );
        bytes32 crossMsgHash = CrossMsgHelper.toHash(crossMsg);
        bytes32 prevMsgsHash = checkpoint.data.crossMsgs.msgsHash;
        bytes32 newMsgsHash = prevMsgsHash;

        if (
            checkpoint.hasCrossMsgMeta() && crossMsgRegistry[epoch].length == 0
        ) {
            require(prevMsgsHash == EMPTY_HASH, "no msgmeta found for cid");
        }

        if (crossMsgExistInRegistry[epoch][crossMsgHash] == false) {
            crossMsgRegistry[epoch].push(crossMsg);

            newMsgsHash = CrossMsgHelper.toHash(crossMsgRegistry[epoch]);
        }

        crossMsgEpochRegistry[prevMsgsHash] = -1;
        crossMsgEpochRegistry[newMsgsHash] = epoch;
        crossMsgCidRegistry[epoch] = newMsgsHash;
        crossMsgExistInRegistry[epoch][crossMsgHash] = true;

        checkpoint.data.crossMsgs.msgsHash = newMsgsHash;
        checkpoint.data.crossMsgs.value += msg.value;
        checkpoint.data.crossMsgs.fee += crossMsgFee;

        nonce += 1;

        payable(BURNT_FUNDS_ACTOR).sendValue(releaseAmount);
    }

    function getSubnet(
        address actor
    ) internal view returns (bool found, Subnet storage subnet) {
        SubnetID memory subnetId = networkName.createSubnetId(actor);

        return getSubnet(subnetId);
    }

    function getSubnet(
        SubnetID memory subnetId
    ) internal view returns (bool found, Subnet storage subnet) {
        subnet = subnets[subnetId.toHash()];
        found = subnet.id.route.length > 0;
    }

    function distributeRewards(address to, uint256 amount) internal {
        Address.functionCallWithValue(
            to.normalize(),
            abi.encodeWithSignature("reward()"),
            amount
        );
    }
}
