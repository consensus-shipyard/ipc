// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import "forge-std/Test.sol";
import "../../contracts/lib/FvmAddressHelper.sol";
import "../../contracts/lib/SubnetIDHelper.sol";
import {BottomUpBatchHelper} from "../helpers/BottomUpBatchHelper.sol";
import {BottomUpBatch} from "../../contracts/structs/BottomUpBatch.sol";
import {FvmAddress} from "../../contracts/structs/FvmAddress.sol";
import {IPCAddress} from "../../contracts/structs/Subnet.sol";
import {IpcEnvelope, IpcMsgKind} from "../../contracts/structs/CrossNet.sol";
import {LibBottomUpBatch} from "../../contracts/lib/LibBottomUpBatch.sol";

contract BottomUpBatchHelperTest is Test {
    using FvmAddressHelper for FvmAddress;
    using SubnetIDHelper for SubnetID;

    function test_batch_empty() public {
        IpcEnvelope[] memory msgs = createCrossMsgs(0);
        verifyValidProofs(msgs);
    }

    function test_batch_one() public {
        IpcEnvelope[] memory msgs = createCrossMsgs(1);
        verifyValidProofs(msgs);
    }

    function test_batch_many() public {
        IpcEnvelope[] memory msgs = createCrossMsgs(10);
        verifyValidProofs(msgs);
    }

    function verifyValidProofs(IpcEnvelope[] memory msgs) internal {
        BottomUpBatch.Commitment memory commitment = BottomUpBatchHelper.makeCommitment(msgs);
        BottomUpBatch.Inclusion[] memory inclusions = BottomUpBatchHelper.makeInclusions(msgs);
        for (uint256 i = 0; i < inclusions.length; i++) {
            BottomUpBatch.MerkleHash leaf = LibBottomUpBatch.makeLeaf(msgs[i]);
            LibBottomUpBatch.ensureValidProof(inclusions[i].proof, commitment.msgsRoot, leaf);
        }
    }

    function createCrossMsgs(uint256 length) internal pure returns (IpcEnvelope[] memory _crossMsgs) {
        _crossMsgs = new IpcEnvelope[](length);

        for (uint64 i = 0; i < length; i++) {
            _crossMsgs[i] = createDefaultTransferMsg(i);
        }
    }

    function createDefaultTransferMsg(uint64 nonce) internal pure returns (IpcEnvelope memory) {
        IPCAddress memory addr = IPCAddress({
            subnetId: SubnetID(0, new address[](0)),
            rawAddress: FvmAddressHelper.from(address(0))
        });
        return createTransferMsg(addr, addr, nonce);
    }

    function createTransferMsg(
        IPCAddress memory from,
        IPCAddress memory to,
        uint64 nonce
    ) internal pure returns (IpcEnvelope memory) {
        return
            IpcEnvelope({
            kind: IpcMsgKind.Transfer,
            from: from,
            to: to,
            value: 0,
            message: bytes(""),
            localNonce: nonce,
            originalNonce: 0
        });
    }
}