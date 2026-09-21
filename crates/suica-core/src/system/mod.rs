use crate::{
    config::{Config, SystemBackendKind},
    error::CoreError,
    mode::DisplayMode,
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

#[async_trait]
pub trait SystemBackend: Send + Sync {
    async fn reconcile(&self) -> Result<DisplayMode, CoreError>;
    async fn start_tv(&self) -> Result<(), CoreError>;
    async fn stop_tv(&self) -> Result<(), CoreError>;
    async fn show_home(&self) -> Result<(), CoreError>;
}

pub struct SimulatedSystemBackend {
    mode: RwLock<DisplayMode>,
}
impl SimulatedSystemBackend {
    pub fn new() -> Self {
        Self {
            mode: RwLock::new(DisplayMode::Tv),
        }
    }
}
impl Default for SimulatedSystemBackend {
    fn default() -> Self {
        Self::new()
    }
}
#[async_trait]
impl SystemBackend for SimulatedSystemBackend {
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
        *self.mode.write().await = DisplayMode::Tv;
        Ok(())
    }
}

pub async fn create_system_backend(config: &Config) -> Result<Arc<dyn SystemBackend>, CoreError> {
    match config.system_backend {
        SystemBackendKind::Simulated => Ok(Arc::new(SimulatedSystemBackend::new())),
        SystemBackendKind::Linux => create_linux(config),
        SystemBackendKind::Auto => {
            #[cfg(target_os = "linux")]
            {
                create_linux(config)
            }
            #[cfg(not(target_os = "linux"))]
            {
                Ok(Arc::new(SimulatedSystemBackend::new()))
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn create_linux(config: &Config) -> Result<Arc<dyn SystemBackend>, CoreError> {
    Ok(Arc::new(linux::LinuxSystemBackend::new(config.clone())))
}
#[cfg(not(target_os = "linux"))]
fn create_linux(_config: &Config) -> Result<Arc<dyn SystemBackend>, CoreError> {
    Err(CoreError::Config(
        "linux backend is only available on Linux".into(),
    ))
}

#[cfg(target_os = "linux")]
mod linux {
    use crate::{config::Config, error::CoreError, mode::DisplayMode, system::SystemBackend};
    use async_trait::async_trait;
    use nix::{
        sys::signal::{Signal, kill},
        unistd::Pid,
    };
    use std::{path::Path, process::Stdio, time::Duration};
    use tokio::{process::Command, time::sleep};

    pub struct LinuxSystemBackend {
        config: Config,
    }
    impl LinuxSystemBackend {
        pub fn new(config: Config) -> Self {
            Self { config }
        }
        async fn managed_pid(&self) -> Result<Option<u32>, CoreError> {
            let raw = match tokio::fs::read_to_string(&self.config.pid_file).await {
                Ok(v) => v,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(e) => return Err(e.into()),
            };
            let pid: u32 = match raw.trim().parse() {
                Ok(v) => v,
                Err(_) => {
                    let _ = tokio::fs::remove_file(&self.config.pid_file).await;
                    return Ok(None);
                }
            };
            let cmd = match tokio::fs::read(format!("/proc/{pid}/cmdline")).await {
                Ok(v) => v,
                Err(_) => {
                    let _ = tokio::fs::remove_file(&self.config.pid_file).await;
                    return Ok(None);
                }
            };
            let marker = format!("--user-data-dir={}", self.config.profile_dir.display());
            if !String::from_utf8_lossy(&cmd).contains(&marker) {
                return Err(CoreError::ProcessControlFailed(
                    "PID does not belong to Suica TV Chromium".into(),
                ));
            }
            Ok(Some(pid))
        }
        async fn write_pid(&self, pid: u32) -> Result<(), CoreError> {
            if let Some(p) = self.config.pid_file.parent() {
                tokio::fs::create_dir_all(p).await?
            }
            let tmp = self.config.pid_file.with_extension("tmp");
            tokio::fs::write(&tmp, pid.to_string()).await?;
            tokio::fs::rename(tmp, &self.config.pid_file).await?;
            Ok(())
        }
        async fn wait_exit(&self, pid: u32, attempts: u8) -> bool {
            for _ in 0..attempts {
                if !Path::new(&format!("/proc/{pid}")).exists() {
                    return true;
                }
                sleep(Duration::from_millis(250)).await;
            }
            false
        }
    }
    #[async_trait]
    impl SystemBackend for LinuxSystemBackend {
        async fn reconcile(&self) -> Result<DisplayMode, CoreError> {
            Ok(if self.managed_pid().await?.is_some() {
                DisplayMode::Tv
            } else {
                DisplayMode::Pc
            })
        }
        async fn start_tv(&self) -> Result<(), CoreError> {
            if self.managed_pid().await?.is_some() {
                return Ok(());
            }
            tokio::fs::create_dir_all(&self.config.profile_dir).await?;
            let child = Command::new(&self.config.chromium_binary)
                .arg("--kiosk")
                .arg("--no-first-run")
                .arg("--disable-session-crashed-bubble")
                .arg("--autoplay-policy=no-user-gesture-required")
                .arg(format!(
                    "--user-data-dir={}",
                    self.config.profile_dir.display()
                ))
                .arg(format!("--app={}", self.config.home_url))
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| CoreError::ProcessControlFailed(e.to_string()))?;
            let pid = child.id().ok_or_else(|| {
                CoreError::ProcessControlFailed("Chromium did not return a PID".into())
            })?;
            self.write_pid(pid).await?;
            sleep(Duration::from_millis(500)).await;
            if !Path::new(&format!("/proc/{pid}")).exists() {
                return Err(CoreError::ProcessControlFailed(
                    "Chromium exited during startup".into(),
                ));
            }
            if let Some(binary) = &self.config.unclutter_binary {
                let _ = Command::new(binary)
                    .args(["-idle", "1", "-root"])
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn();
            }
            Ok(())
        }
        async fn stop_tv(&self) -> Result<(), CoreError> {
            let Some(pid) = self.managed_pid().await? else {
                return Ok(());
            };
            kill(Pid::from_raw(pid as i32), Signal::SIGTERM)
                .map_err(|e| CoreError::ProcessControlFailed(e.to_string()))?;
            if !self.wait_exit(pid, 20).await {
                kill(Pid::from_raw(pid as i32), Signal::SIGKILL)
                    .map_err(|e| CoreError::ProcessControlFailed(e.to_string()))?;
                if !self.wait_exit(pid, 4).await {
                    return Err(CoreError::ProcessControlFailed(
                        "Chromium did not exit".into(),
                    ));
                }
            }
            let _ = tokio::fs::remove_file(&self.config.pid_file).await;
            Ok(())
        }
        async fn show_home(&self) -> Result<(), CoreError> {
            self.stop_tv().await?;
            self.start_tv().await
        }
    }
}
