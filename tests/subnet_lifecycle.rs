// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::common::TestClient;
use ipc_sdk::subnet_id::{SubnetID, ROOTNET_ID};

mod common;

const IPC_AGENT_JSON_RPC_URL_ENV: &str = "IPC_AGENT_JSON_RPC_URL";

#[tokio::test]
async fn subnet_lifecycle() {
    let client = TestClient::new(std::env::var(IPC_AGENT_JSON_RPC_URL_ENV).ok());

    // step 1. create the subnet
    let address = client
        .create_subnet("/root")
        .await
        .expect("create subnet in root failed");

    // obtain the created subnet id
    let subnet_id = SubnetID::new_from_parent(&ROOTNET_ID, address);
    log::info!("created subnet: {:} in root", subnet_id);

    // step 2. join the subnet
    client
        .join_subnet(&subnet_id, String::from("test_validator"))
        .await
        .expect("cannot join subnet");
    log::info!("joined subnet: {:}", subnet_id);

    // step 3. try kill the subnet, fail because not all validators have left
    let r = client.kill_subnet(&subnet_id).await;
    assert!(
        r.is_err(),
        "should failed when killing subnet as not all validators have left"
    );
    log::info!(
        "expected cannot kill subnet: {:} when there are validators in subnet",
        subnet_id
    );

    // step 4. leave the subnet
    client
        .leave_subnet(&subnet_id)
        .await
        .expect("cannot leave subnet");
    log::info!("left subnet: {:}", subnet_id);

    // step 5. kill the subnet works now as all validators have left
    client
        .kill_subnet(&subnet_id)
        .await
        .expect("cannot kill subnet");
}
