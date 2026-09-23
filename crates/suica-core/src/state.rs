use crate::{
    config::Config,
    error::CoreError,
    mode::ModeManager,
    pairing::{FailureLimiter, TokenStore},
    remote::ServerMessage,
};
use std::{
    collections::{HashMap, HashSet},
    net::IpAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{Mutex, Notify, RwLock, mpsc};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClientRole {
    Remote,
    Tv,
}
impl ClientRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Remote => "remote",
            Self::Tv => "tv",
        }
    }
}
#[derive(Clone)]
struct ClientEntry {
    role: ClientRole,
    tx: mpsc::Sender<ServerMessage>,
}
#[derive(Clone, Default)]
pub struct ClientRegistry {
    inner: Arc<RwLock<HashMap<Uuid, ClientEntry>>>,
}
impl ClientRegistry {
    pub async fn has_tv(&self) -> bool {
        self.inner
            .read()
            .await
            .values()
            .any(|client| client.role == ClientRole::Tv)
    }

    pub async fn register(
        &self,
        role: ClientRole,
    ) -> Result<(Uuid, mpsc::Receiver<ServerMessage>), CoreError> {
        let mut all = self.inner.write().await;
        let count = all.values().filter(|c| c.role == role).count();
        if role == ClientRole::Remote && count >= 4 {
            return Err(CoreError::ForbiddenRole);
        }
        // A home navigation restarts Chromium. Replace the stale TV connection so the
        // newly loaded React application can reconnect without waiting for socket cleanup.
        if role == ClientRole::Tv {
            all.retain(|_, client| client.role != ClientRole::Tv);
        }
        let id = Uuid::new_v4();
        let (tx, rx) = mpsc::channel(32);
        all.insert(id, ClientEntry { role, tx });
        Ok((id, rx))
    }
    pub async fn remove(&self, id: Uuid) {
        self.inner.write().await.remove(&id);
    }
    pub async fn send_tv(&self, msg: ServerMessage) -> Result<(), CoreError> {
        let tx = self
            .inner
            .read()
            .await
            .values()
            .find(|c| c.role == ClientRole::Tv)
            .map(|c| c.tx.clone())
            .ok_or(CoreError::TvClientUnavailable)?;
        tx.try_send(msg).map_err(|_| CoreError::TvClientUnavailable)
    }
    pub async fn broadcast(&self, msg: ServerMessage) {
        let senders: Vec<_> = self
            .inner
            .read()
            .await
            .values()
            .map(|c| c.tx.clone())
            .collect();
        for tx in senders {
            let _ = tx.try_send(msg.clone());
        }
    }
}

#[derive(Default)]
struct RequestCacheInner {
    completed: HashMap<Uuid, (Instant, ServerMessage)>,
    in_flight: HashSet<Uuid>,
}

pub enum RequestDecision {
    Execute,
    Cached(ServerMessage),
}

#[derive(Clone, Default)]
pub struct RequestCache {
    inner: Arc<Mutex<RequestCacheInner>>,
    changed: Arc<Notify>,
}
impl RequestCache {
    pub async fn begin(&self, id: Uuid) -> RequestDecision {
        loop {
            let notified = self.changed.notified();
            {
                let mut inner = self.inner.lock().await;
                let cutoff = Instant::now() - Duration::from_secs(60);
                inner.completed.retain(|_, (t, _)| *t > cutoff);
                if let Some((_, message)) = inner.completed.get(&id) {
                    return RequestDecision::Cached(message.clone());
                }
                if inner.in_flight.insert(id) {
                    return RequestDecision::Execute;
                }
            }
            notified.await;
        }
    }
    pub async fn finish(&self, id: Uuid, msg: ServerMessage) {
        let mut inner = self.inner.lock().await;
        if inner.completed.len() >= 1000
            && let Some(old) = inner
                .completed
                .iter()
                .min_by_key(|(_, v)| v.0)
                .map(|(k, _)| *k)
        {
            inner.completed.remove(&old);
        }
        inner.completed.insert(id, (Instant::now(), msg));
        inner.in_flight.remove(&id);
        drop(inner);
        self.changed.notify_waiters();
    }
}

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub clients: ClientRegistry,
    pub mode_manager: Arc<ModeManager>,
    pub token_store: Arc<TokenStore>,
    pub requests: RequestCache,
    pub pairing_failures: Arc<FailureLimiter>,
}

pub struct CommandRateLimiter {
    tokens: f64,
    last: Instant,
}
impl Default for CommandRateLimiter {
    fn default() -> Self {
        Self {
            tokens: 40.0,
            last: Instant::now(),
        }
    }
}
impl CommandRateLimiter {
    pub fn allow(&mut self) -> bool {
        let now = Instant::now();
        self.tokens = (self.tokens + now.duration_since(self.last).as_secs_f64() * 20.0).min(40.0);
        self.last = now;
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

pub fn is_loopback(ip: IpAddr) -> bool {
    ip.is_loopback()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn remote_connection_limits_are_released_on_remove() {
        let clients = ClientRegistry::default();
        let mut remotes = Vec::new();
        for _ in 0..4 {
            remotes.push(clients.register(ClientRole::Remote).await.unwrap().0);
        }
        assert!(clients.register(ClientRole::Remote).await.is_err());
        clients.remove(remotes[0]).await;
        assert!(clients.register(ClientRole::Remote).await.is_ok());
    }

    #[tokio::test]
    async fn new_tv_connection_replaces_the_previous_connection() {
        let clients = ClientRegistry::default();
        let (_, mut previous) = clients.register(ClientRole::Tv).await.unwrap();
        let (_, mut current) = clients.register(ClientRole::Tv).await.unwrap();

        assert!(previous.recv().await.is_none());
        let message = ServerMessage::Shutdown {
            retry_after_seconds: 3,
        };
        clients.send_tv(message).await.unwrap();
        assert!(matches!(
            current.recv().await,
            Some(ServerMessage::Shutdown {
                retry_after_seconds: 3
            })
        ));
    }

    #[tokio::test]
    async fn reports_whether_a_tv_client_is_connected() {
        let clients = ClientRegistry::default();
        assert!(!clients.has_tv().await);
        let (id, _) = clients.register(ClientRole::Tv).await.unwrap();
        assert!(clients.has_tv().await);
        clients.remove(id).await;
        assert!(!clients.has_tv().await);
    }

    #[tokio::test]
    async fn duplicate_request_waits_for_original_result() {
        let cache = RequestCache::default();
        let id = Uuid::new_v4();
        assert!(matches!(cache.begin(id).await, RequestDecision::Execute));
        let waiting = {
            let cache = cache.clone();
            tokio::spawn(async move { cache.begin(id).await })
        };
        tokio::task::yield_now().await;
        assert!(!waiting.is_finished());
        let result = ServerMessage::CommandResult {
            request_id: id,
            ok: true,
            error: None,
        };
        cache.finish(id, result).await;
        assert!(matches!(
            waiting.await.unwrap(),
            RequestDecision::Cached(ServerMessage::CommandResult { ok: true, .. })
        ));
    }

    #[test]
    fn rate_limiter_allows_burst_then_rejects() {
        let mut limiter = CommandRateLimiter::default();
        assert!((0..40).all(|_| limiter.allow()));
        assert!(!limiter.allow());
    }
}
