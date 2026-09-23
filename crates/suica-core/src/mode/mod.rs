use crate::{
    error::CoreError,
    system::{BrowserKey, SystemBackend},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DisplayMode {
    Tv,
    Pc,
}
#[derive(Clone, Debug)]
pub enum ModeState {
    Initializing,
    Stable(DisplayMode),
    Switching {
        from: DisplayMode,
        to: DisplayMode,
        request_id: Uuid,
    },
    Degraded {
        last_stable: Option<DisplayMode>,
        reason: String,
    },
}
pub struct ModeSnapshot {
    pub mode: DisplayMode,
    pub transitioning: bool,
    pub target: Option<DisplayMode>,
    pub changed_at: DateTime<Utc>,
}
pub struct ModeManager {
    backend: Arc<dyn SystemBackend>,
    state: RwLock<ModeState>,
    transition: Mutex<()>,
    changed_at: RwLock<DateTime<Utc>>,
}
impl ModeManager {
    pub async fn new(backend: Arc<dyn SystemBackend>) -> Result<Self, CoreError> {
        let initial = backend.reconcile().await?;
        Ok(Self {
            backend,
            state: RwLock::new(ModeState::Stable(initial)),
            transition: Mutex::new(()),
            changed_at: RwLock::new(Utc::now()),
        })
    }
    pub async fn snapshot(&self) -> ModeSnapshot {
        let state = self.state.read().await;
        let changed = *self.changed_at.read().await;
        match &*state {
            ModeState::Stable(m) => ModeSnapshot {
                mode: *m,
                transitioning: false,
                target: None,
                changed_at: changed,
            },
            ModeState::Switching { from, to, .. } => ModeSnapshot {
                mode: *from,
                transitioning: true,
                target: Some(*to),
                changed_at: changed,
            },
            ModeState::Degraded { last_stable, .. } => ModeSnapshot {
                mode: last_stable.unwrap_or(DisplayMode::Pc),
                transitioning: false,
                target: None,
                changed_at: changed,
            },
            ModeState::Initializing => ModeSnapshot {
                mode: DisplayMode::Pc,
                transitioning: true,
                target: Some(DisplayMode::Tv),
                changed_at: changed,
            },
        }
    }
    pub async fn switch(
        &self,
        target: DisplayMode,
        request_id: Uuid,
    ) -> Result<DisplayMode, CoreError> {
        let _guard = self.transition.try_lock().map_err(|_| CoreError::Busy)?;
        let from = self.snapshot().await.mode;
        if from == target {
            return Ok(target);
        }
        *self.state.write().await = ModeState::Switching {
            from,
            to: target,
            request_id,
        };
        let result = match target {
            DisplayMode::Tv => self.backend.start_tv().await,
            DisplayMode::Pc => self.backend.stop_tv().await,
        };
        match result {
            Ok(()) => {
                *self.state.write().await = ModeState::Stable(target);
                *self.changed_at.write().await = Utc::now();
                Ok(target)
            }
            Err(e) => {
                *self.state.write().await = ModeState::Stable(from);
                Err(e)
            }
        }
    }
    pub async fn ensure_tv_home(&self, request_id: Uuid) -> Result<(), CoreError> {
        if self.snapshot().await.mode != DisplayMode::Tv {
            self.switch(DisplayMode::Tv, request_id).await?;
        }
        self.backend.show_home().await
    }
    pub async fn send_browser_key(&self, key: BrowserKey) -> Result<(), CoreError> {
        self.backend.send_browser_key(key).await
    }
    pub async fn type_browser_text(&self, text: &str) -> Result<(), CoreError> {
        self.backend.type_browser_text(text).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    struct Fake {
        mode: RwLock<DisplayMode>,
    }
    #[async_trait]
    impl SystemBackend for Fake {
        async fn reconcile(&self) -> Result<DisplayMode, CoreError> {
            Ok(*self.mode.read().await)
        }
        async fn start_tv(&self) -> Result<(), CoreError> {
            *self.mode.write().await = DisplayMode::Tv;
            Ok(())
        }
        async fn stop_tv(&self) -> Result<(), CoreError> {
            *self.mode.write().await = DisplayMode::Pc;
            Ok(())
        }
        async fn show_home(&self) -> Result<(), CoreError> {
            Ok(())
        }
        async fn send_browser_key(&self, _key: BrowserKey) -> Result<(), CoreError> {
            Ok(())
        }
        async fn type_browser_text(&self, _text: &str) -> Result<(), CoreError> {
            Ok(())
        }
    }
    #[tokio::test]
    async fn switches_both_ways() {
        let m = ModeManager::new(Arc::new(Fake {
            mode: RwLock::new(DisplayMode::Tv),
        }))
        .await
        .unwrap();
        assert_eq!(
            m.switch(DisplayMode::Pc, Uuid::new_v4()).await.unwrap(),
            DisplayMode::Pc
        );
        assert_eq!(
            m.switch(DisplayMode::Tv, Uuid::new_v4()).await.unwrap(),
            DisplayMode::Tv
        );
    }

    struct Failing;
    #[async_trait]
    impl SystemBackend for Failing {
        async fn reconcile(&self) -> Result<DisplayMode, CoreError> {
            Ok(DisplayMode::Tv)
        }
        async fn start_tv(&self) -> Result<(), CoreError> {
            Ok(())
        }
        async fn stop_tv(&self) -> Result<(), CoreError> {
            Err(CoreError::ModeSwitchFailed("test".into()))
        }
        async fn show_home(&self) -> Result<(), CoreError> {
            Ok(())
        }
        async fn send_browser_key(&self, _key: BrowserKey) -> Result<(), CoreError> {
            Ok(())
        }
        async fn type_browser_text(&self, _text: &str) -> Result<(), CoreError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn failed_switch_preserves_old_mode() {
        let manager = ModeManager::new(Arc::new(Failing)).await.unwrap();
        assert!(
            manager
                .switch(DisplayMode::Pc, Uuid::new_v4())
                .await
                .is_err()
        );
        assert_eq!(manager.snapshot().await.mode, DisplayMode::Tv);
    }

    struct Slow;
    #[async_trait]
    impl SystemBackend for Slow {
        async fn reconcile(&self) -> Result<DisplayMode, CoreError> {
            Ok(DisplayMode::Tv)
        }
        async fn start_tv(&self) -> Result<(), CoreError> {
            Ok(())
        }
        async fn stop_tv(&self) -> Result<(), CoreError> {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            Ok(())
        }
        async fn show_home(&self) -> Result<(), CoreError> {
            Ok(())
        }
        async fn send_browser_key(&self, _key: BrowserKey) -> Result<(), CoreError> {
            Ok(())
        }
        async fn type_browser_text(&self, _text: &str) -> Result<(), CoreError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn concurrent_switch_returns_busy() {
        let manager = Arc::new(ModeManager::new(Arc::new(Slow)).await.unwrap());
        let first = {
            let manager = manager.clone();
            tokio::spawn(async move { manager.switch(DisplayMode::Pc, Uuid::new_v4()).await })
        };
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        assert!(matches!(
            manager.switch(DisplayMode::Pc, Uuid::new_v4()).await,
            Err(CoreError::Busy)
        ));
        first.await.unwrap().unwrap();
    }
}
