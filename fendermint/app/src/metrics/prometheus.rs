// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Prometheus metrics

macro_rules! metrics {
        ($($name:ident : $type:ty = $desc:literal);* $(;)?) => {
            $(
              paste! {
                lazy_static! {
                    pub static ref $name: $type = $type::new(stringify!([< $name:lower >]), $desc).unwrap();
                }
              }
            )*

            pub fn register_metrics(registry: &Registry) -> anyhow::Result<()> {
                $(registry.register(Box::new($name.clone()))?;)*
                Ok(())
            }
        };
    }

/// Metrics emitted by fendermint.
pub mod app {
    use lazy_static::lazy_static;
    use paste::paste;
    use prometheus::{IntCounter, IntGauge, Registry};

    metrics! {
        TOPDOWN_VIEW_BLOCK_HEIGHT: IntGauge = "Highest parent subnet block observed";
        TOPDOWN_VIEW_NUM_MSGS: IntCounter = "Number of top-down messages observed since start";
        TOPDOWN_VIEW_NUM_VAL_CHNGS: IntCounter = "Number of top-down validator changes observed since start";
        TOPDOWN_FINALIZED_BLOCK_HEIGHT: IntGauge = "Highest parent subnet block finalized";

        BOTTOMUP_CKPT_BLOCK_HEIGHT: IntGauge = "Highest bottom-up checkpoint created";
        BOTTOMUP_CKPT_CONFIG_NUM: IntGauge = "Highest configuration number checkpointed";
        BOTTOMUP_CKPT_NUM_MSGS: IntCounter = "Number of bottom-up messages observed since start";

        // This metrics is available in CometBFT as well, but it's something that should increase even without subnets,
        // which can be a useful way to check if metrics work at all.
        ABCI_COMMITTED_BLOCK_HEIGHT: IntGauge = "Highest committed block";
    }
}

/// Metrics emitted by the Ethereum API facade.
pub mod eth {
    // TODO: Define Ethereum metrics and events.
}

#[cfg(test)]
mod tests {
    #[test]
    fn can_register_metrics() {
        let r = prometheus::Registry::new();
        super::app::register_metrics(&r).unwrap();
    }
}
