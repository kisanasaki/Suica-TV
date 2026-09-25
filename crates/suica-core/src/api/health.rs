//! プロセスの生存性とTV静的ファイルの準備状態を公開する。
//!
//! liveはプロセス応答、readyは配信可能性を表し、外部サービスの状態までは保証しない。

use crate::state::AppState;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde_json::json;

pub async fn live() -> Json<serde_json::Value> {
    Json(json!({"status":"ok","service":"suica-core"}))
}
pub async fn ready(State(state): State<AppState>) -> impl IntoResponse {
    if state.config.static_dir.join("index.html").is_file() {
        (StatusCode::OK, Json(json!({"status":"ready"})))
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"status":"not_ready","reason":"static_files_unavailable"})),
        )
    }
}
