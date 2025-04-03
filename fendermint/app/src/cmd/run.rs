// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::{cmd, options::run::RunArgs};
use fendermint_app::service::node::run as run_node;

cmd! {
  RunArgs(self, settings) {
    run_node(settings, None).await
  }
}
