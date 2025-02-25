use async_stm::atomically;
use fendermint_vm_message::chain::ChainMessage;
use fendermint_vm_message::ipc::{BottomUpCheckpoint, CertifiedMessage, IpcMessage};
use fendermint_vm_resolver::pool::{ResolveKey, ResolvePool};

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum CheckpointPoolItem {
    /// BottomUp checkpoints to be resolved from the originating subnet or the current one.
    BottomUp(CertifiedMessage<BottomUpCheckpoint>),
    // We can extend this to include top-down checkpoints as well, with slightly
    // different resolution semantics (resolving it from a trusted parent, and
    // awaiting finality before declaring it available).
}

impl From<CertifiedMessage<BottomUpCheckpoint>> for CheckpointPoolItem {
    fn from(value: CertifiedMessage<BottomUpCheckpoint>) -> Self {
        CheckpointPoolItem::BottomUp(value)
    }
}

impl From<&CheckpointPoolItem> for ResolveKey {
    fn from(value: &CheckpointPoolItem) -> Self {
        match value {
            CheckpointPoolItem::BottomUp(cp) => {
                (cp.message.subnet_id.clone(), cp.message.bottom_up_messages)
            }
        }
    }
}

pub struct BottomUpCheckpointResolver {
    pool: ResolvePool<CheckpointPoolItem>,
}

impl BottomUpCheckpointResolver {
    pub fn new(resolve_pool: ResolvePool<CheckpointPoolItem>) -> Self {
        Self { pool: resolve_pool }
    }

    pub async fn check_checkpoint_resolved(
        &self,
        msg: CertifiedMessage<BottomUpCheckpoint>,
    ) -> bool {
        let item = CheckpointPoolItem::BottomUp(msg);

        // We can just look in memory because when we start the application, we should retrieve any
        // pending checkpoints (relayed but not executed) from the ledger, so they should be there.
        // We don't have to validate the checkpoint here, because
        // 1) we validated it when it was relayed, and
        // 2) if a validator proposes something invalid, we can make them pay during execution.
        let is_resolved = atomically(|| match self.pool.get_status(&item)? {
            None => Ok(false),
            Some(status) => status.is_resolved(),
        })
        .await;
        is_resolved
    }

    // Checks the bottom up checkpoint pool and returns the messages that are ready for execution
    pub async fn messages_from_resolved_checkpoints(&self) -> Vec<ChainMessage> {
        let resolved = atomically(|| self.pool.collect_resolved()).await;
        resolved
            .into_iter()
            .map(|checkpoint| match checkpoint {
                CheckpointPoolItem::BottomUp(checkpoint) => {
                    ChainMessage::Ipc(IpcMessage::BottomUpExec(checkpoint))
                }
            })
            .collect()
    }
}
