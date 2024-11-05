use async_trait::async_trait;
use ethers::core::types::{transaction::eip2718::TypedTransaction, *};
use ethers::providers::{Middleware, MiddlewareError, PendingTransaction};
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;

#[derive(Debug)]
/// Middleware used for calculating nonces locally, useful for signing multiple
/// consecutive transactions without waiting for them to hit the mempool
pub struct NonceManagerMiddleware<M> {
    inner: M,
    nonce: AtomicU64,
    address: Address,
}

impl<M> NonceManagerMiddleware<M>
where
    M: Middleware,
{
    /// Instantiates the nonce manager with a 0 nonce. The `address` should be the
    /// address which you'll be sending transactions from
    pub fn new(inner: M, address: Address) -> Self {
        Self {
            inner,
            nonce: Default::default(),
            address,
        }
    }

    /// Returns the next nonce to be used
    pub fn next(&self) -> U256 {
        let nonce = self.nonce.fetch_add(1, Ordering::SeqCst);
        nonce.into()
    }

    pub async fn initialize_nonce(&self) -> Result<U256, NonceManagerError<M>> {
        let nonce = self
            .inner
            .get_transaction_count(self.address, Some(BlockId::Number(BlockNumber::Pending)))
            .await
            .map_err(MiddlewareError::from_err)?;
        self.nonce.store(nonce.as_u64(), Ordering::SeqCst);
        Ok(nonce)
    }

    async fn get_transaction_count_with_manager(&self) -> Result<U256, NonceManagerError<M>> {
        if self.nonce.load(Ordering::SeqCst) == 0 {
            self.initialize_nonce().await?;
        }
        Ok(self.next())
    }
}

#[derive(Error, Debug)]
/// Thrown when an error happens at the Nonce Manager
pub enum NonceManagerError<M: Middleware> {
    /// Thrown when the internal middleware errors
    #[error("{0}")]
    MiddlewareError(M::Error),
}

impl<M: Middleware> MiddlewareError for NonceManagerError<M> {
    type Inner = M::Error;

    fn from_err(src: M::Error) -> Self {
        NonceManagerError::MiddlewareError(src)
    }

    fn as_inner(&self) -> Option<&Self::Inner> {
        match self {
            NonceManagerError::MiddlewareError(e) => Some(e),
        }
    }
}

#[async_trait]
impl<M> Middleware for NonceManagerMiddleware<M>
where
    M: Middleware,
{
    type Error = NonceManagerError<M>;
    type Provider = M::Provider;
    type Inner = M;

    fn inner(&self) -> &M {
        &self.inner
    }

    async fn fill_transaction(
        &self,
        tx: &mut TypedTransaction,
        block: Option<BlockId>,
    ) -> Result<(), Self::Error> {
        if tx.nonce().is_none() {
            tx.set_nonce(self.get_transaction_count_with_manager().await?);
        }

        Ok(self
            .inner()
            .fill_transaction(tx, block)
            .await
            .map_err(MiddlewareError::from_err)?)
    }

    async fn send_transaction<T: Into<TypedTransaction> + Send + Sync>(
        &self,
        tx: T,
        block: Option<BlockId>,
    ) -> Result<PendingTransaction<'_, Self::Provider>, Self::Error> {
        let mut tx = tx.into();

        if tx.nonce().is_none() {
            tx.set_nonce(self.get_transaction_count_with_manager().await?);
        }

        self.inner
            .send_transaction(tx, block)
            .await
            .map_err(MiddlewareError::from_err)
    }
}
