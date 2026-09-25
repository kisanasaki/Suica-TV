//! 6桁コードによる端末ペアリングHTTP APIを提供する。
//!
//! 端末名と試行回数を検証し、成功時だけ一度限りの平文トークンを返す。
//! トークンの保存方式はTokenStoreへ委譲する。

use crate::{error::CoreError, pairing::valid_pairing_code, state::AppState};
use axum::{
    Json,
    extract::{ConnectInfo, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PairRequest {
    code: String,
    device_name: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PairResponse {
    device_id: uuid::Uuid,
    token: String,
}
pub async fn pair(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(body): Json<PairRequest>,
) -> Result<impl IntoResponse, CoreError> {
    if !state.pairing_failures.allowed(addr.ip()).await {
        return Err(CoreError::Unauthorized);
    }
    if body.device_name.trim().is_empty() || body.device_name.chars().count() > 64 {
        return Err(CoreError::InvalidMessage);
    }
    if !valid_pairing_code(&state.config.pairing_code, &body.code) {
        state.pairing_failures.record(addr.ip()).await;
        return Err(CoreError::Unauthorized);
    }
    let (id, token) = state
        .token_store
        .issue(body.device_name.trim().to_string())
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(PairResponse {
            device_id: id,
            token,
        }),
    ))
}
