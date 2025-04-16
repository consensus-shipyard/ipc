// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_shared::econ::TokenAmount;
use ipc_provider::config::serialize::{
    vec_to_allow_headers, vec_to_allow_methods, vec_to_allow_origin,
};
use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize};
use serde_with::{serde_as, DurationSeconds};
use std::time::Duration;
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin};

use ipc_observability::config::TracingSettings;

use crate::{IsHumanReadable, MetricsSettings, SocketAddress};

/// Ethereum API facade settings.
#[serde_as]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EthSettings {
    pub listen: SocketAddress,
    #[serde_as(as = "DurationSeconds<u64>")]
    pub filter_timeout: Duration,
    pub cache_capacity: usize,
    pub gas: GasOpt,
    pub max_nonce_gap: u64,
    pub metrics: MetricsSettings,
    pub cors: CorsOpt,
    pub tracing: TracingSettings,
}

impl Default for EthSettings {
    fn default() -> Self {
        EthSettings {
            filter_timeout: Duration::from_secs(300),
            cache_capacity: 1000000,
            max_nonce_gap: 10,
            gas: GasOpt {
                min_gas_premium: TokenAmount::from_atto(100000),
                num_blocks_max_prio_fee: 10,
                max_fee_hist_size: 1024,
            },
            listen: SocketAddress {
                host: "127.0.0.1".into(),
                port: 8545,
            },
            metrics: MetricsSettings {
                enabled: true,
                listen: SocketAddress {
                    host: "127.0.0.1".into(),
                    port: 9185,
                },
            },
            cors: CorsOpt::default(),
            tracing: TracingSettings::default(),
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasOpt {
    /// Minimum gas fee in atto.
    #[serde_as(as = "IsHumanReadable")]
    pub min_gas_premium: TokenAmount,
    pub num_blocks_max_prio_fee: u64,
    pub max_fee_hist_size: u64,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Default)]
pub struct CorsOpt {
    #[serde(default, rename = "allowed_origins")]
    pub temp_origins: Vec<String>,
    #[serde(default, rename = "allowed_methods")]
    pub temp_methods: Vec<String>,
    #[serde(default, rename = "allowed_headers")]
    pub temp_headers: Vec<String>,

    // Runtime representations, skipped during (de)serialization.
    #[serde(skip)]
    pub allowed_origins: AllowOrigin,
    #[serde(skip)]
    pub allowed_methods: AllowMethods,
    #[serde(skip)]
    pub allowed_headers: AllowHeaders,
}

impl<'de> Deserialize<'de> for CorsOpt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Temp {
            #[serde(default, rename = "allowed_origins")]
            allowed_origins: Vec<String>,
            #[serde(default, rename = "allowed_methods")]
            allowed_methods: Vec<String>,
            #[serde(default, rename = "allowed_headers")]
            allowed_headers: Vec<String>,
        }

        let temp = Temp::deserialize(deserializer)?;

        let allowed_origins =
            vec_to_allow_origin(temp.allowed_origins.clone()).map_err(D::Error::custom)?;
        let allowed_methods =
            vec_to_allow_methods(temp.allowed_methods.clone()).map_err(D::Error::custom)?;
        let allowed_headers =
            vec_to_allow_headers(temp.allowed_headers.clone()).map_err(D::Error::custom)?;

        Ok(CorsOpt {
            temp_origins: temp.allowed_origins,
            temp_methods: temp.allowed_methods,
            temp_headers: temp.allowed_headers,
            allowed_origins,
            allowed_methods,
            allowed_headers,
        })
    }
}
