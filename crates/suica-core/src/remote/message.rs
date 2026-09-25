//! Coreからクライアントへ送るWebSocketメッセージを定義する。
//!
//! すべての送信箇所で同じcamelCase形式と安全なエラー本文を使うため、
//! JSON生成をServerMessageへ集約する。

use crate::mode::DisplayMode;
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "server.hello", rename_all = "camelCase")]
    Hello {
        protocol_version: u8,
        server_version: String,
        connection_id: Uuid,
        role: String,
    },
    #[serde(rename = "system.state", rename_all = "camelCase")]
    SystemState {
        mode: DisplayMode,
        transitioning: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        target_mode: Option<DisplayMode>,
        changed_at: DateTime<Utc>,
    },
    #[serde(rename = "remote.command", rename_all = "camelCase")]
    RemoteCommand { request_id: Uuid, action: String },
    #[serde(rename = "command.result", rename_all = "camelCase")]
    CommandResult {
        request_id: Uuid,
        ok: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<ErrorBody>,
    },
    #[serde(rename = "error")]
    Error { error: ErrorBody },
    #[serde(rename = "server.shutdown", rename_all = "camelCase")]
    Shutdown { retry_after_seconds: u8 },
}

#[derive(Clone, Debug, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}
impl ErrorBody {
    pub fn from_error(e: &crate::error::CoreError) -> Self {
        Self {
            code: e.code().into(),
            message: e.safe_message().into(),
            retryable: e.retryable(),
        }
    }
}
impl ServerMessage {
    pub fn text(&self) -> String {
        serde_json::to_string(self).expect("server messages are serializable")
    }
}
