use crate::error::CoreError;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    net::IpAddr,
    path::PathBuf,
    time::{Duration, Instant},
};
use subtle::ConstantTimeEq;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenRecord {
    device_id: Uuid,
    device_name: String,
    token_hash: String,
    #[serde(default)]
    created_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PairedDevice {
    pub device_id: Uuid,
    pub device_name: String,
    pub created_at: Option<DateTime<Utc>>,
}
pub struct TokenStore {
    path: PathBuf,
    records: RwLock<Vec<TokenRecord>>,
}
impl TokenStore {
    pub async fn load(path: PathBuf) -> Result<Self, CoreError> {
        let records = match tokio::fs::read(&path).await {
            Ok(bytes) => serde_json::from_slice(&bytes)?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(e) => return Err(e.into()),
        };
        Ok(Self {
            path,
            records: RwLock::new(records),
        })
    }
    pub async fn issue(&self, device_name: String) -> Result<(Uuid, String), CoreError> {
        let mut bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut bytes);
        let token = URL_SAFE_NO_PAD.encode(bytes);
        let id = Uuid::new_v4();
        let mut records = self.records.write().await;
        let mut updated = records.clone();
        updated.push(TokenRecord {
            device_id: id,
            device_name,
            token_hash: hash(&token),
            created_at: Some(Utc::now()),
        });
        self.persist(&updated).await?;
        *records = updated;
        Ok((id, token))
    }
    pub async fn verify(&self, token: &str) -> bool {
        self.verify_device(token).await.is_some()
    }
    pub async fn verify_device(&self, token: &str) -> Option<Uuid> {
        let wanted = hash(token);
        self.records
            .read()
            .await
            .iter()
            .find(|r| bool::from(r.token_hash.as_bytes().ct_eq(wanted.as_bytes())))
            .map(|record| record.device_id)
    }
    pub async fn list(&self) -> Vec<PairedDevice> {
        self.records
            .read()
            .await
            .iter()
            .map(|record| PairedDevice {
                device_id: record.device_id,
                device_name: record.device_name.clone(),
                created_at: record.created_at,
            })
            .collect()
    }
    pub async fn revoke(&self, device_id: Uuid) -> Result<bool, CoreError> {
        let mut records = self.records.write().await;
        if !records.iter().any(|record| record.device_id == device_id) {
            return Ok(false);
        }
        let mut updated = records.clone();
        updated.retain(|record| record.device_id != device_id);
        self.persist(&updated).await?;
        *records = updated;
        Ok(true)
    }
    async fn persist(&self, records: &[TokenRecord]) -> Result<(), CoreError> {
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let data = serde_json::to_vec_pretty(records)?;
        let tmp = self.path.with_extension("tmp");
        tokio::fs::write(&tmp, data).await?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            tokio::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600)).await?;
        }
        tokio::fs::rename(tmp, &self.path).await?;
        Ok(())
    }
}
fn hash(value: &str) -> String {
    let mut h = Sha256::new();
    h.update(value.as_bytes());
    format!("{:x}", h.finalize())
}

#[derive(Default)]
pub struct FailureLimiter {
    entries: RwLock<HashMap<IpAddr, Vec<Instant>>>,
}
impl FailureLimiter {
    pub async fn allowed(&self, ip: IpAddr) -> bool {
        let cutoff = Instant::now() - Duration::from_secs(60);
        let mut all = self.entries.write().await;
        let e = all.entry(ip).or_default();
        e.retain(|t| *t > cutoff);
        e.len() < 10
    }
    pub async fn record(&self, ip: IpAddr) {
        self.entries
            .write()
            .await
            .entry(ip)
            .or_default()
            .push(Instant::now());
    }
}

pub fn valid_pairing_code(expected: &str, provided: &str) -> bool {
    expected.len() == provided.len() && bool::from(expected.as_bytes().ct_eq(provided.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    #[tokio::test]
    async fn tokens_survive_reload() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("tokens.json");
        let s = TokenStore::load(p.clone()).await.unwrap();
        let (_, t) = s.issue("phone".into()).await.unwrap();
        assert!(s.verify(&t).await);
        drop(s);
        assert!(TokenStore::load(p).await.unwrap().verify(&t).await);
    }
    #[tokio::test]
    async fn token_file_never_contains_plain_token() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("tokens.json");
        let s = TokenStore::load(p.clone()).await.unwrap();
        let (_, token) = s.issue("phone".into()).await.unwrap();
        let stored = tokio::fs::read_to_string(p).await.unwrap();
        assert!(!stored.contains(&token));
        assert!(stored.contains("tokenHash"));
    }
    #[tokio::test]
    async fn concurrent_issues_are_all_persisted() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("tokens.json");
        let store = Arc::new(TokenStore::load(p.clone()).await.unwrap());
        let mut tasks = Vec::new();
        for index in 0..16 {
            let store = store.clone();
            tasks.push(tokio::spawn(async move {
                store.issue(format!("phone-{index}")).await.unwrap().1
            }));
        }
        let mut tokens = Vec::new();
        for task in tasks {
            tokens.push(task.await.unwrap());
        }

        let reloaded = TokenStore::load(p).await.unwrap();
        for token in tokens {
            assert!(reloaded.verify(&token).await);
        }
    }
    #[tokio::test]
    async fn failed_persist_does_not_publish_record_in_memory() {
        let d = tempfile::tempdir().unwrap();
        let parent_file = d.path().join("not-a-directory");
        let store = TokenStore::load(parent_file.join("tokens.json"))
            .await
            .unwrap();
        tokio::fs::write(&parent_file, b"occupied").await.unwrap();

        assert!(store.issue("phone".into()).await.is_err());
        assert!(store.records.read().await.is_empty());
    }
    #[tokio::test]
    async fn lists_and_revokes_only_the_selected_device() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("tokens.json");
        let store = TokenStore::load(p.clone()).await.unwrap();
        let (first_id, first_token) = store.issue("first phone".into()).await.unwrap();
        let (second_id, second_token) = store.issue("second phone".into()).await.unwrap();

        let devices = store.list().await;
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].device_id, first_id);
        assert_eq!(devices[0].device_name, "first phone");
        assert!(devices[0].created_at.is_some());

        assert!(store.revoke(first_id).await.unwrap());
        assert!(!store.verify(&first_token).await);
        assert!(store.verify(&second_token).await);
        assert!(!store.revoke(first_id).await.unwrap());

        let reloaded = TokenStore::load(p).await.unwrap();
        assert_eq!(reloaded.list().await[0].device_id, second_id);
    }

    #[tokio::test]
    async fn loads_legacy_records_without_created_at() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("tokens.json");
        let device_id = Uuid::new_v4();
        tokio::fs::write(
            &p,
            format!(
                r#"[{{"deviceId":"{device_id}","deviceName":"legacy","tokenHash":"{}"}}]"#,
                hash("legacy-token")
            ),
        )
        .await
        .unwrap();

        let store = TokenStore::load(p).await.unwrap();
        assert_eq!(store.list().await[0].created_at, None);
        assert_eq!(store.verify_device("legacy-token").await, Some(device_id));
    }
    #[tokio::test]
    async fn failure_limiter_stops_at_ten_attempts() {
        let limiter = FailureLimiter::default();
        let ip: IpAddr = "192.0.2.1".parse().unwrap();
        for _ in 0..10 {
            assert!(limiter.allowed(ip).await);
            limiter.record(ip).await;
        }
        assert!(!limiter.allowed(ip).await);
    }
    #[test]
    fn code_check() {
        assert!(valid_pairing_code("123456", "123456"));
        assert!(!valid_pairing_code("123456", "123457"));
    }
}
