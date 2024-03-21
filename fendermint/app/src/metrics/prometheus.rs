// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Prometheus metrics
use lazy_static::lazy_static;
use paste::paste;
use prometheus::{IntGauge, Registry};

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

metrics! {

    TOPDOWN_VIEW_BLOCK_HEIGHT: IntGauge = "Highest parent subnet block observed";
    TOPDOWN_FINALIZED_BLOCK_HEIGHT: IntGauge = "Highest parent subnet block finalized";

}
