use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("invalid message")]
    InvalidMessage,
    #[error("invalid command")]
    InvalidCommand,
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden role")]
    ForbiddenRole,
    #[error("unsupported protocol")]
    UnsupportedProtocol,
    #[error("operation is busy")]
    Busy,
    #[error("TV client is unavailable")]
    TvClientUnavailable,
    #[error("invalid state")]
    InvalidState,
    #[error("operation timed out")]
    Timeout,
    #[error("mode switch failed: {0}")]
    ModeSwitchFailed(String),
    #[error("process control failed: {0}")]
    ProcessControlFailed(String),
    #[error("configuration error: {0}")]
    Config(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Toml(#[from] toml::de::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("internal error: {0}")]
    Internal(String),
}

impl CoreError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidMessage => "invalid_message",
            Self::InvalidCommand => "invalid_command",
            Self::Unauthorized => "unauthorized",
            Self::ForbiddenRole => "forbidden_role",
            Self::UnsupportedProtocol => "unsupported_protocol",
            Self::Busy => "busy",
            Self::TvClientUnavailable => "tv_client_unavailable",
            Self::InvalidState => "invalid_state",
            Self::Timeout => "timeout",
            Self::ModeSwitchFailed(_) | Self::ProcessControlFailed(_) => "mode_switch_failed",
            _ => "internal_error",
        }
    }
    pub fn retryable(&self) -> bool {
        matches!(
            self,
            Self::Busy
                | Self::TvClientUnavailable
                | Self::InvalidState
                | Self::Timeout
                | Self::ModeSwitchFailed(_)
                | Self::ProcessControlFailed(_)
                | Self::Internal(_)
        )
    }
    pub fn safe_message(&self) -> &'static str {
        match self {
            Self::InvalidMessage => "メッセージ形式が不正です。",
            Self::InvalidCommand => "未対応の操作です。",
            Self::Unauthorized => "認証できませんでした。",
            Self::ForbiddenRole => "この接続では操作できません。",
            Self::UnsupportedProtocol => "プロトコルバージョンに対応していません。",
            Self::Busy => "モード切り替えを実行中です。",
            Self::TvClientUnavailable => "TV画面へ接続できません。",
            Self::InvalidState => "現在の状態では操作できません。",
            Self::Timeout => "操作がタイムアウトしました。",
            Self::ModeSwitchFailed(_) | Self::ProcessControlFailed(_) => {
                "表示モードを切り替えられませんでした。"
            }
            _ => "内部エラーが発生しました。",
        }
    }
}

impl IntoResponse for CoreError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::ForbiddenRole => StatusCode::FORBIDDEN,
            Self::InvalidMessage
            | Self::InvalidCommand
            | Self::UnsupportedProtocol
            | Self::Config(_) => StatusCode::BAD_REQUEST,
            Self::Busy => StatusCode::CONFLICT,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(json!({"error":{"code":self.code(),"message":self.safe_message(),"retryable":self.retryable()}}))).into_response()
    }
}
