use crate::services::Service;
use anyhow::Result;
use async_trait::async_trait;
use fendermint_app::service::node::run as run_node;
use fendermint_app_settings::Settings;
use tokio_util::sync::CancellationToken;

pub struct NodeService {
    settings: Settings,
}

impl NodeService {
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }
}

#[async_trait]
impl Service for NodeService {
    fn name(&self) -> &'static str {
        "Node Service"
    }

    async fn run(&self, shutdown: CancellationToken) -> Result<()> {
        run_node(self.settings.clone(), Some(shutdown)).await
    }
}
