// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use std::io::Write;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, Condvar, Mutex};

use fvm_shared::address::Address;
use indoc::formatdoc;
use ipc_sdk::subnet_id::{SubnetID, ROOTNET_ID};
use tempfile::NamedTempFile;
use url::Url;

use crate::config::{Config, ReloadableConfig};

// Arguments for the config's fields
const SERVER_JSON_RPC_ADDR: &str = "127.0.0.1:3030";
const ROOT_ID: &str = "/root";
const CHILD_ID: &str = "/root/f0100";
const ROOT_AUTH_TOKEN: &str = "ROOT_AUTH_TOKEN";
const CHILD_AUTH_TOKEN: &str = "CHILD_AUTH_TOKEN";
const JSONRPC_API_HTTP: &str = "https://example.org/rpc/v0";
const JSONRPC_API_WS: &str = "ws://example.org/rpc/v0";
const ACCOUNT_ADDRESS: &str =
    "f3thgjtvoi65yzdcoifgqh6utjbaod3ukidxrx34heu34d6avx6z7r5766t5jqt42a44ehzcnw3u5ehz47n42a";

#[tokio::test]
async fn reload_works() {
    let config_str = config_str();

    let mut file = NamedTempFile::new().unwrap();
    let path = file
        .path()
        .as_os_str()
        .to_os_string()
        .into_string()
        .unwrap();

    file.write_all(config_str.as_bytes()).unwrap();

    let h = Arc::new(ReloadableConfig::new(path.clone()).unwrap());
    let original_config = h.get_config();

    // A simple barrier implementation for testing.
    // Refer to: https://web.mit.edu/rust-lang_v1.25/arch/amd64_ubuntu1404/share/doc/rust/html/std/sync/struct.Condvar.html#examples
    // Only when the main thread has created a new subscriber then we trigger then update the config file.
    // This way, we dont miss the update and stuck the main thread.
    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let pair2 = pair.clone();
    let h_cloned = h.clone();
    tokio::spawn(async move {
        {
            let &(ref lock, ref cvar) = &*pair;
            let mut started = lock.lock().unwrap();
            while !*started {
                started = cvar.wait(started).unwrap();
            }
        };

        let config_str = config_str_diff_addr();

        let mut file = file.reopen().unwrap();
        file.write_all(config_str.as_bytes()).unwrap();

        h_cloned.reload(path).await.unwrap();
    });

    let mut rx = h.new_subscriber();
    {
        let &(ref lock, ref cvar) = &*pair2;
        let mut started = lock.lock().unwrap();
        *started = true;
        cvar.notify_one();
    }
    rx.recv().await.unwrap();

    let updated_config = h.get_config();
    assert_ne!(
        updated_config.server.json_rpc_address,
        original_config.server.json_rpc_address
    );
}

#[test]
fn check_server_config() {
    let config = read_config().server;
    assert_eq!(
        config.json_rpc_address,
        SocketAddr::from_str(SERVER_JSON_RPC_ADDR).unwrap(),
        "invalid server rpc address"
    );
}

#[test]
fn check_subnets_config() {
    let config = read_config().subnets;

    let root = &config["root"];
    assert_eq!(root.id, *ROOTNET_ID);
    assert_eq!(
        root.jsonrpc_api_http,
        Url::from_str(JSONRPC_API_HTTP).unwrap()
    );
    assert_eq!(
        root.jsonrpc_api_ws.as_ref().unwrap(),
        &Url::from_str(JSONRPC_API_WS).unwrap()
    );
    assert_eq!(root.auth_token.as_ref().unwrap(), ROOT_AUTH_TOKEN);

    let child = &config["child"];
    assert_eq!(child.id, SubnetID::from_str(CHILD_ID).unwrap(),);
    assert_eq!(
        child.jsonrpc_api_http,
        Url::from_str(JSONRPC_API_HTTP).unwrap(),
    );
    assert_eq!(child.auth_token.as_ref().unwrap(), CHILD_AUTH_TOKEN,);
    assert_eq!(
        child.accounts.as_ref(),
        vec![Address::from_str(ACCOUNT_ADDRESS).unwrap()],
    );
}

fn config_str() -> String {
    formatdoc!(
        r#"
            [server]
            json_rpc_address = "{SERVER_JSON_RPC_ADDR}"
            network = 0

            [subnets]
            [subnets.root]
            id = "{ROOT_ID}"
            jsonrpc_api_http = "{JSONRPC_API_HTTP}"
            jsonrpc_api_ws = "{JSONRPC_API_WS}"
            auth_token = "{ROOT_AUTH_TOKEN}"
            [subnets.child]
            id = "{CHILD_ID}"
            jsonrpc_api_http = "{JSONRPC_API_HTTP}"
            auth_token = "{CHILD_AUTH_TOKEN}"
            accounts = ["{ACCOUNT_ADDRESS}"]
        "#
    )
}

fn config_str_diff_addr() -> String {
    formatdoc!(
        r#"
            [server]
            json_rpc_address = "127.0.0.1:3031"
            network = 0

            [subnets]
            [subnets.root]
            id = "{ROOT_ID}"
            jsonrpc_api_http = "{JSONRPC_API_HTTP}"
            jsonrpc_api_ws = "{JSONRPC_API_WS}"
            auth_token = "{ROOT_AUTH_TOKEN}"
            [subnets.child]
            id = "{CHILD_ID}"
            jsonrpc_api_http = "{JSONRPC_API_HTTP}"
            auth_token = "{CHILD_AUTH_TOKEN}"
            accounts = ["{ACCOUNT_ADDRESS}"]
        "#
    )
}

fn read_config() -> Config {
    let config_str = formatdoc!(
        r#"
            [server]
            json_rpc_address = "{SERVER_JSON_RPC_ADDR}"
            network = 0

            [subnets]

            [subnets.root]
            id = "{ROOT_ID}"
            jsonrpc_api_http = "{JSONRPC_API_HTTP}"
            jsonrpc_api_ws = "{JSONRPC_API_WS}"
            auth_token = "{ROOT_AUTH_TOKEN}"

            [subnets.child]
            id = "{CHILD_ID}"
            jsonrpc_api_http = "{JSONRPC_API_HTTP}"
            auth_token = "{CHILD_AUTH_TOKEN}"
            accounts = ["{ACCOUNT_ADDRESS}"]
        "#
    );

    Config::from_toml_str(config_str.as_str()).unwrap()
}
