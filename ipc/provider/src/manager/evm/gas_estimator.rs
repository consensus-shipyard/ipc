// This estimator is from Alloy library.
// TODO migrate to Alloy from ethers eventually.

// The number of blocks from the past for which the fee rewards are fetched for fee estimation.
pub const EIP1559_FEE_ESTIMATION_PAST_BLOCKS: u64 = 10;
/// Multiplier for the current base fee to estimate max base fee for the next block.
pub const EIP1559_BASE_FEE_MULTIPLIER: u128 = 2;
/// The default percentile of gas premiums that are fetched for fee estimation.
pub const EIP1559_FEE_ESTIMATION_REWARD_PERCENTILE: f64 = 20.0;
/// The minimum priority fee to provide.
pub const EIP1559_MIN_PRIORITY_FEE: u128 = 1;

/// Return type of EIP1155 gas fee estimator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Eip1559Estimation {
    /// The base fee per gas.
    pub max_fee_per_gas: u128,
    /// The max priority fee per gas.
    pub max_priority_fee_per_gas: u128,
}

fn estimate_priority_fee(rewards: &[Vec<u128>]) -> u128 {
    let mut rewards = rewards
        .iter()
        .filter_map(|r| r.first())
        .filter(|r| **r > 0_u128)
        .collect::<Vec<_>>();
    if rewards.is_empty() {
        return EIP1559_MIN_PRIORITY_FEE;
    }

    rewards.sort_unstable();

    let n = rewards.len();

    let median = if n % 2 == 0 {
        (*rewards[n / 2 - 1] + *rewards[n / 2]) / 2
    } else {
        *rewards[n / 2]
    };

    std::cmp::max(median, EIP1559_MIN_PRIORITY_FEE)
}

/// Based on the work by [MetaMask](https://github.com/MetaMask/core/blob/main/packages/gas-fee-controller/src/fetchGasEstimatesViaEthFeeHistory/calculateGasFeeEstimatesForPriorityLevels.ts#L56);
/// constants for "medium" priority level are used.
pub fn eip1559_estimator(base_fee_per_gas: u128, rewards: &[Vec<u128>]) -> Eip1559Estimation {
    let max_priority_fee_per_gas = estimate_priority_fee(rewards);
    let potential_max_fee = base_fee_per_gas * EIP1559_BASE_FEE_MULTIPLIER;

    Eip1559Estimation {
        max_fee_per_gas: potential_max_fee + max_priority_fee_per_gas,
        max_priority_fee_per_gas,
    }
}
