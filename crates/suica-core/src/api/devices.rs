use crate::{
    error::CoreError,
    pairing::PairedDevice,
    state::{AppState, is_loopback},
};
use axum::{
    Json,
    extract::{ConnectInfo, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_json::json;
use std::net::SocketAddr;
use uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DevicesResponse {
    devices: Vec<PairedDevice>,
}

fn ensure_loopback(addr: SocketAddr) -> Result<(), CoreError> {
    if is_loopback(addr.ip()) {
        Ok(())
    } else {
        Err(CoreError::ForbiddenRole)
    }
}

pub async fn list(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Result<Json<DevicesResponse>, CoreError> {
    ensure_loopback(addr)?;
    Ok(Json(DevicesResponse {
        devices: state.token_store.list().await,
    }))
}

pub async fn revoke(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(device_id): Path<Uuid>,
) -> Result<Response, CoreError> {
    ensure_loopback(addr)?;
    if !state.token_store.revoke(device_id).await? {
        return Ok((
            StatusCode::NOT_FOUND,
            Json(json!({"error": {
                "code": "device_not_found",
                "message": "登録端末が見つかりません。",
                "retryable": false
            }})),
        )
            .into_response());
    }
    state.clients.disconnect_device(device_id).await;
    Ok(StatusCode::NO_CONTENT.into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_management_is_loopback_only() {
        assert!(ensure_loopback("127.0.0.1:1234".parse().unwrap()).is_ok());
        assert!(ensure_loopback("[::1]:1234".parse().unwrap()).is_ok());
        assert!(matches!(
            ensure_loopback("192.0.2.1:1234".parse().unwrap()),
            Err(CoreError::ForbiddenRole)
        ));
    }
}
