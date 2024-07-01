// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

// See https://ethereum.org/en/developers/docs/apis/json-rpc/#json-rpc-methods
// and https://ethereum.github.io/execution-apis/api-documentation/

use crate::HybridClient;
use jsonrpc_v2::{Factory, MapRouter, ServerBuilder};
use lazy_static::lazy_static;
use prometheus::{register_histogram_vec, HistogramVec};
use std::marker::PhantomData;

mod eth;
mod net;
mod web3;

pub struct Timer<S, E, T, F: Factory<S, E, T>> {
    pub factory: F,
    ph: PhantomData<(S, E, T)>,
}

impl<S, E, T, F: Factory<S, E, T>> Timer<S, E, T, F> {
    pub fn new(f: F) -> Self {
        Self {
            factory: f,
            ph: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl<S: Sync, E: Sync, T: Sync + Send, F: Factory<S, E, T> + Sync> Factory<S, E, T>
    for Timer<S, E, T, F>
{
    async fn call(&self, param: T) -> Result<S, E> {
        let timer = RPC_METHOD_CALL_LATENCY_SECONDS
            .with_label_values(&[stringify!([< $module _ $method >])])
            .start_timer();
        let result = self.factory.call(param).await;
        timer.observe_duration();
        result
    }
}

macro_rules! with_methods_timer {
    ($server:ident, $module:ident, { $($method:ident),* }) => {
        paste::paste! {
            $server
                $(
                    .with_method(
                        stringify!([< $module _ $method >]),
                        Timer::new($module::[< $method:snake >]::<HybridClient>)
                    )
                )*
        }
    };
}

// TODO - move this to a more appropriate place - perhaps in the metrics module?
lazy_static! {
    pub static ref RPC_METHOD_CALL_LATENCY_SECONDS: HistogramVec = register_histogram_vec!(
        "rpc_method_call_duration_seconds",
        "Histogram of RPC method call durations",
        &["method"]
    )
    .unwrap();
}

// TODO - find a better way to generate these methods ideally with single macro or without macros.
macro_rules! with_methods_one_arg {
    ($server:ident, $module:ident, { $($method:ident),* }) => {
        paste::paste! {
            $server
                $(
                    .with_method(
                        stringify!([< $module _ $method >]),
                        |arg| {
                            async move {
                                let timer = RPC_METHOD_CALL_LATENCY_SECONDS
                                    .with_label_values(&[stringify!([< $module _ $method >])])
                                    .start_timer();
                                let result = $module::[< $method:snake >]::<HybridClient>(arg).await;
                                timer.observe_duration();
                                result
                            }
                        }
                    )
                )*
        }
    };
}

macro_rules! with_methods_two_args {
    ($server:ident, $module:ident, { $($method:ident),* }) => {
        paste::paste! {
            $server
                $(
                    .with_method(
                        stringify!([< $module _ $method >]),
                        |arg1, arg2| {
                            async move {
                                let timer = RPC_METHOD_CALL_LATENCY_SECONDS
                                    .with_label_values(&[stringify!([< $module _ $method >])])
                                    .start_timer();
                                let result = $module::[< $method:snake >]::<HybridClient>(arg1, arg2).await;
                                timer.observe_duration();
                                result
                            }
                        }
                    )
                )*
        }
    };
}

pub fn register_methods(server: ServerBuilder<MapRouter>) -> ServerBuilder<MapRouter> {
    // This is the list of eth methods. Apart from these Lotus implements 1 method from web3,
    // while Ethermint does more across web3, debug, miner, net, txpool, and personal.
    // The unimplemented ones are commented out, to make it easier to see where we're at.

    /*
        TODO - add missing methods:
        // eth_coinbase
        // eth_compileLLL
        // eth_compileSerpent
        // eth_compileSolidity
        // eth_getCompilers
        // eth_getWork
        // eth_hashrate
        // eth_mining
        // eth_sendTransaction
        // eth_sign
        // eth_signTransaction
        // eth_submitHashrate
        // eth_submitWork
    */

    let server = with_methods_timer!(server, eth, {
        accounts,
        blockNumber,
        call
    });

    let server = with_methods_one_arg!(server, eth, {
        chainId,
        maxPriorityFeePerGas,
        gasPrice,
        newBlockFilter,
        newPendingTransactionFilter,
        protocolVersion,
        syncing
    });

    let server = with_methods_two_args!(server, eth, {
        estimateGas,
        feeHistory,
        getBalance,
        getBlockByHash,
        getBlockByNumber,
        getBlockReceipts,
        getCode,
        getFilterChanges,
        getFilterLogs,
        getLogs,
        getStorageAt,
        getBlockTransactionCountByNumber,
        getBlockTransactionCountByHash,
        getTransactionByBlockHashAndIndex,
        getTransactionByBlockNumberAndIndex,
        getTransactionByHash,
        getTransactionCount,
        getTransactionReceipt,
        getUncleByBlockHashAndIndex,
        getUncleByBlockNumberAndIndex,
        getUncleCountByBlockHash,
        getUncleCountByBlockNumber,
        newFilter,
        sendRawTransaction,
        sendRawTransaction,
        uninstallFilter,
        subscribe,
        unsubscribe
    });

    let server = with_methods_one_arg!(server, web3, { clientVersion });
    let server = with_methods_two_args!(server, web3, { sha3 });

    with_methods_one_arg!(server, net, {
        version,
        listening,
        peerCount
    })
}

/// Indicate whether a method requires a WebSocket connection.
pub fn is_streaming_method(method: &str) -> bool {
    method == "eth_subscribe"
}
