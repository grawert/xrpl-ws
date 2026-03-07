use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::broadcast;
use tokio::task;
use crate::XrplClient;
use crate::error::XrplError;

/// Handle for managing an active XRPL subscription.
pub struct SubscriptionHandle<T, U>
where
    T: Clone + Send + Sync + 'static + crate::request::XrplSubscription,
    U: Clone + Send + 'static,
{
    client: Arc<XrplClient>,
    params: T,
    receiver: broadcast::Receiver<U>,
    unsubscribed: AtomicBool,
}

impl<T, U> SubscriptionHandle<T, U>
where
    T: Clone + Send + Sync + 'static + crate::request::XrplSubscription,
    U: Clone + Send + 'static,
{
    pub(crate) fn new(
        client: Arc<XrplClient>,
        params: T,
        receiver: broadcast::Receiver<U>,
    ) -> Self {
        Self { client, params, receiver, unsubscribed: AtomicBool::new(false) }
    }

    /// Get a mutable reference to the event receiver for this subscription.
    pub fn receiver(&mut self) -> &mut broadcast::Receiver<U> {
        &mut self.receiver
    }

    /// Unsubscribe from this subscription. This is also called automatically on drop.
    pub async fn unsubscribe(&self) -> Result<(), XrplError> {
        if self.unsubscribed.swap(true, Ordering::SeqCst) {
            return Ok(());
        }
        self.client.unsubscribe(&self.params).await
    }
}

impl<T, U> Drop for SubscriptionHandle<T, U>
where
    T: Clone + Send + Sync + 'static + crate::request::XrplSubscription,
    U: Clone + Send + 'static,
{
    fn drop(&mut self) {
        if self.unsubscribed.load(Ordering::SeqCst) {
            return;
        }
        let client = self.client.clone();
        let params = self.params.clone();
        let unsubscribed =
            std::mem::replace(&mut self.unsubscribed, AtomicBool::new(true));
        task::spawn(async move {
            if unsubscribed.swap(true, Ordering::SeqCst) {
                return;
            }
            let _ = client.unsubscribe(&params).await;
        });
    }
}
