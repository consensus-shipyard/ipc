//! Monitoring of networking interfaces and route changes.

use n0_future::{
    boxed::BoxFuture,
    task::{self, AbortOnDropHandle},
};
use nested_enum_utils::common_fields;
use snafu::{Backtrace, ResultExt, Snafu};
use tokio::sync::{mpsc, oneshot};

mod actor;
#[cfg(target_os = "android")]
mod android;
#[cfg(any(
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "macos",
    target_os = "ios"
))]
mod bsd;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(wasm_browser)]
mod wasm_browser;
#[cfg(target_os = "windows")]
mod windows;

pub use self::actor::CallbackToken;
use self::actor::{Actor, ActorMessage};

/// Monitors networking interface and route changes.
#[derive(Debug)]
pub struct Monitor {
    /// Task handle for the monitor task.
    _handle: AbortOnDropHandle<()>,
    actor_tx: mpsc::Sender<ActorMessage>,
}

#[common_fields({
    backtrace: Option<Backtrace>,
})]
#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum Error {
    #[snafu(display("channel closed"))]
    ChannelClosed {},
    #[snafu(display("actor error"))]
    Actor { source: actor::Error },
}

impl<T> From<mpsc::error::SendError<T>> for Error {
    fn from(_value: mpsc::error::SendError<T>) -> Self {
        ChannelClosedSnafu.build()
    }
}

impl From<oneshot::error::RecvError> for Error {
    fn from(_value: oneshot::error::RecvError) -> Self {
        ChannelClosedSnafu.build()
    }
}

impl Monitor {
    /// Create a new monitor.
    pub async fn new() -> Result<Self, Error> {
        let actor = Actor::new().await.context(ActorSnafu)?;
        let actor_tx = actor.subscribe();

        let handle = task::spawn(async move {
            actor.run().await;
        });

        Ok(Monitor {
            _handle: AbortOnDropHandle::new(handle),
            actor_tx,
        })
    }

    /// Subscribe to network changes.
    pub async fn subscribe<F>(&self, callback: F) -> Result<CallbackToken, Error>
    where
        F: Fn(bool) -> BoxFuture<()> + 'static + Sync + Send,
    {
        let (s, r) = oneshot::channel();
        self.actor_tx
            .send(ActorMessage::Subscribe(Box::new(callback), s))
            .await?;
        let token = r.await?;
        Ok(token)
    }

    /// Unsubscribe a callback from network changes, using the provided token.
    pub async fn unsubscribe(&self, token: CallbackToken) -> Result<(), Error> {
        let (s, r) = oneshot::channel();
        self.actor_tx
            .send(ActorMessage::Unsubscribe(token, s))
            .await?;
        r.await?;
        Ok(())
    }

    /// Potential change detected outside
    pub async fn network_change(&self) -> Result<(), Error> {
        self.actor_tx.send(ActorMessage::NetworkChange).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use n0_future::future::FutureExt;

    use super::*;

    #[tokio::test]
    async fn test_smoke_monitor() {
        let mon = Monitor::new().await.unwrap();
        let _token = mon
            .subscribe(|is_major| {
                async move {
                    println!("CHANGE DETECTED: {}", is_major);
                }
                .boxed()
            })
            .await
            .unwrap();

        tokio::time::sleep(std::time::Duration::from_secs(15)).await;
    }
}
