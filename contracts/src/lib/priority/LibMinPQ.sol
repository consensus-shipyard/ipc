// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {LibValidatorSet} from "../LibStaking.sol";
import {ValidatorSet} from "../../structs/Subnet.sol";
import {PQ, LibPQ} from "./LibPQ.sol";

struct MinPQ {
    PQ inner;
}

/// The min index priority queue for staking
library LibMinPQ {
    using LibPQ for PQ;
    using LibValidatorSet for ValidatorSet;

    function getSize(MinPQ storage self) internal view returns (uint16) {
        return self.inner.size;
    }

    function getAddress(MinPQ storage self, uint16 i) internal view returns (address) {
        return self.inner.posToAddress[i];
    }

    function contains(MinPQ storage self, address validator) internal view returns (bool) {
        return self.inner.contains(validator);
    }

    /// @notice Insert the validator address into this PQ.
    /// NOTE that caller should ensure the validator is not already in the queue.
    function insert(MinPQ storage self, ValidatorSet storage validators, address validator) internal {
        uint16 size = self.inner.size + 1;

        self.inner.addressToPos[validator] = size;
        self.inner.posToAddress[size] = validator;

        self.inner.size = size;

        uint256 confirmedCollateral = validators.getConfirmedCollateral(validator);
        swim({self: self, validators: validators, pos: size, value: confirmedCollateral});
    }

    /// @notice Pop the minimal value in the priority queue.
    function pop(MinPQ storage self, ValidatorSet storage validators) internal {
        self.inner.requireNotEmpty();

        uint16 size = self.inner.size;

        self.inner.exchange(1, size);

        self.inner.size = size - 1;
        self.inner.del(size);

        uint256 value = self.inner.getConfirmedCollateral(validators, 1);
        sink({self: self, validators: validators, pos: 1, value: value});
    }

    /// @notice Reheapify the heap when the validator is deleted.
    function deleteReheapify(MinPQ storage self, ValidatorSet storage validators, address validator) internal {
        uint16 pos = self.inner.getPosOrRevert(validator);
        uint16 size = self.inner.size;

        self.inner.exchange(pos, size);

        // remove the item
        self.inner.size = size - 1;
        self.inner.del(size);

        if (size == pos) {
            return;
        }

        // swim pos up in case exchanged index is smaller
        uint256 val = self.inner.getConfirmedCollateral(validators, pos);
        swim({self: self, validators: validators, pos: pos, value: val});

        // sink pos down in case updated pos is larger
        val = self.inner.getConfirmedCollateral(validators, pos);
        sink({self: self, validators: validators, pos: pos, value: val});
    }

    /// @notice Reheapify the heap when the collateral of a key has increased.
    function increaseReheapify(MinPQ storage self, ValidatorSet storage validators, address validator) internal {
        uint16 pos = self.inner.getPosOrRevert(validator);
        uint256 val = validators.getConfirmedCollateral(validator);
        sink({self: self, validators: validators, pos: pos, value: val});
    }

    /// @notice Reheapify the heap when the collateral of a key has decreased.
    function decreaseReheapify(MinPQ storage self, ValidatorSet storage validators, address validator) internal {
        uint16 pos = self.inner.getPosOrRevert(validator);
        uint256 val = validators.getConfirmedCollateral(validator);
        swim({self: self, validators: validators, pos: pos, value: val});
    }

    /// @notice Get the minimal value in the priority queue.
    /// NOTE that caller should ensure the queue is not empty!
    function min(MinPQ storage self, ValidatorSet storage validators) internal view returns (address, uint256) {
        self.inner.requireNotEmpty();

        address addr = self.inner.posToAddress[1];
        uint256 collateral = validators.getConfirmedCollateral(addr);
        return (addr, collateral);
    }

    /***************************************************************************
     * Heap internal helper functions, should not be called by external functions
     ****************************************************************************/
    function swim(MinPQ storage self, ValidatorSet storage validators, uint16 pos, uint256 value) internal {
        uint16 parentPos;
        uint256 parentCollateral;

        while (pos > 1) {
            // parentPos = pos / 2;
            parentPos = pos >> 1;
            parentCollateral = self.inner.getConfirmedCollateral(validators, parentPos);

            // parent collateral is not more than that of the current child, heap condition met.
            if (!firstValueLarger(parentCollateral, value)) {
                break;
            }

            self.inner.exchange(parentPos, pos);
            pos = parentPos;
        }
    }

    function sink(MinPQ storage self, ValidatorSet storage validators, uint16 pos, uint256 value) internal {
        uint16 childPos = pos * 2;
        uint256 childCollateral;

        uint16 size = self.inner.size;

        while (childPos <= size) {
            if (childPos < size) {
                // select the min of the two children
                (childPos, childCollateral) = smallerPosition({
                    self: self,
                    validators: validators,
                    pos1: childPos,
                    pos2: childPos + 1
                });
            } else {
                childCollateral = self.inner.getConfirmedCollateral(validators, childPos);
            }

            // parent, current idx, is not more than its two children, min heap condition is met.
            if (!firstValueLarger(value, childCollateral)) {
                break;
            }

            self.inner.exchange(childPos, pos);
            pos = childPos;
            childPos = pos * 2;
        }
    }

    /// @notice Get the smaller index of pos1 and pos2.
    function smallerPosition(
        MinPQ storage self,
        ValidatorSet storage validators,
        uint16 pos1,
        uint16 pos2
    ) internal view returns (uint16, uint256) {
        uint256 value1 = self.inner.getConfirmedCollateral(validators, pos1);
        uint256 value2 = self.inner.getConfirmedCollateral(validators, pos2);

        if (!firstValueLarger(value1, value2)) {
            return (pos1, value1);
        }
        return (pos2, value2);
    }

    function firstValueLarger(uint256 v1, uint256 v2) internal pure returns (bool) {
        return v1 > v2;
    }
}
