// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::{cmd, options::run::RunArgs};

cmd! {
  RunArgs(self, settings) {
    crate::service::node::run(settings, None).await
  }
}
