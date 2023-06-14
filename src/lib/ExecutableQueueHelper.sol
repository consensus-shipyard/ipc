// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.18;

import "../structs/ExecutableQueue.sol";

library ExecutableQueueHelper {
    function push(ExecutableQueue storage queue, uint64 epoch) public {
        if (epoch == 0) return;

        if (queue.first == 0 || queue.first > epoch) {
            queue.first = epoch;
        }
        if (queue.last == 0 || queue.last < epoch) {
            queue.last = epoch;
        }

        queue.epochs[epoch] = true;
    }

    function remove(ExecutableQueue storage queue, uint64 epoch) public {
        if (!contains(queue, epoch)) return;

        delete queue.epochs[epoch];

        // there is one element only, so delete everything and exit
        if (queue.first == queue.last) {
            delete queue.first;
            delete queue.last;

            return;
        }

        // epoch is somewhere in the middle, so do nothing else
        if (epoch > queue.first) {
            if (epoch < queue.last) {
                return;
            }
        }

        // find the closest epoch on the right and set it as the new first
        if (epoch == queue.first) {
            uint64 newFirst = queue.first + queue.period;

            while (newFirst < queue.last && !contains(queue, newFirst)) {
                newFirst += queue.period;
            }

            queue.first = newFirst;
        }

        // find the closest epoch on the left and set it as the new last
        if (epoch == queue.last) {
            uint64 newLast = queue.last - queue.period;

            while (newLast > queue.first && !contains(queue, newLast)) {
                newLast -= queue.period;
            }

            queue.last = newLast;
        }
    }

    function contains(ExecutableQueue storage queue, uint64 epoch) public view returns (bool) {
        return queue.epochs[epoch];
    }
}
