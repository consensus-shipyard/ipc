// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

// See https://ethereum.org/en/developers/docs/apis/json-rpc/#json-rpc-methods
// and https://ethereum.github.io/execution-apis/api-documentation/

use jsonrpc_v2::{MapRouter, ServerBuilder};
use paste::paste;
use tendermint_rpc::HttpClient;

mod eth;

macro_rules! with_methods {
    ($server:ident, $module:ident, { $($method:ident),* }) => {
        paste!{
            $server
                $(.with_method(
                    stringify!([< $module _ $method >]),
                    $module :: [< $method:snake >] ::<HttpClient>
                ))*
        }
    };
}

pub fn register_methods(server: ServerBuilder<MapRouter>) -> ServerBuilder<MapRouter> {
    // This is the list of eth methods. Apart from these Lotus implements 1 method from web3,
    // while Ethermint does more across web3, debug, miner, net, txpool, and personal.
    // The unimplemented ones are commented out, to make it easier to see where we're at.
    with_methods!(server, eth, {
        accounts,
        blockNumber,
        // eth_call
        chainId,
        // eth_coinbase
        // eth_compileLLL
        // eth_compileSerpent
        // eth_compileSolidity
        // eth_estimateGas
        feeHistory,
        gasPrice,
        getBalance,
        getBlockByHash,
        getBlockByNumber,
        getBlockTransactionCountByHash,
        getBlockTransactionCountByNumber,
        // eth_getCode
        // eth_getCompilers
        // eth_getFilterChanges
        // eth_getFilterLogs
        // eth_getLogs
        // eth_getStorageAt
        getTransactionByBlockHashAndIndex,
        getTransactionByBlockNumberAndIndex,
        getTransactionByHash,
        getTransactionCount,
        getTransactionReceipt,
        getUncleByBlockHashAndIndex,
        getUncleByBlockNumberAndIndex,
        getUncleCountByBlockHash,
        getUncleCountByBlockNumber,
        // eth_getWork
        // eth_hashrate
        // eth_mining
        // eth_newBlockFilter
        // eth_newFilter
        // eth_newPendingTransactionFilter
        // eth_protocolVersion
        sendRawTransaction
        // eth_sendTransaction
        // eth_sign
        // eth_signTransaction
        // eth_submitHashrate
        // eth_submitWork
        // eth_syncing
        // eth_uninstallFilter
    })
}
