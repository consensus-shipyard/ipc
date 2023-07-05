// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

// See https://ethereum.org/en/developers/docs/apis/json-rpc/#json-rpc-methods
// and https://ethereum.github.io/execution-apis/api-documentation/

use jsonrpc_v2::{MapRouter, ServerBuilder};
use paste::paste;
use tendermint_rpc::HttpClient;

mod eth;
mod net;
mod web3;

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
    let server = with_methods!(server, eth, {
        accounts,
        blockNumber,
        call,
        chainId,
        // eth_coinbase
        // eth_compileLLL
        // eth_compileSerpent
        // eth_compileSolidity
        estimateGas,
        feeHistory,
        gasPrice,
        getBalance,
        getBlockByHash,
        getBlockByNumber,
        getBlockTransactionCountByHash,
        getBlockTransactionCountByNumber,
        getBlockReceipts,
        getCode,
        // eth_getCompilers
        // eth_getFilterChanges
        // eth_getFilterLogs
        getLogs,
        getStorageAt,
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
        protocolVersion,
        sendRawTransaction,
        // eth_sendTransaction
        // eth_sign
        // eth_signTransaction
        // eth_submitHashrate
        // eth_submitWork
        syncing
        // eth_uninstallFilter
    });

    let server = with_methods!(server, web3, {
        clientVersion,
        sha3
    });

    with_methods!(server, net, {
        version,
        listening,
        peerCount
    })
}
