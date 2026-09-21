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
