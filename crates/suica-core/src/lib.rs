//! Suica Coreの構成要素を初期化し、HTTP/WebSocketルーターを組み立てる。
//!
//! 個別の認証、表示制御、メッセージ処理は各モジュールへ委譲し、
//! このファイルは依存関係の接続と公開エンドポイントの定義だけを担当する。

pub mod api;
pub mod config;
pub mod error;
pub mod mode;
pub mod pairing;
pub mod remote;
pub mod state;
pub mod system;

use std::sync::Arc;

use axum::{
    Router,
    routing::{any, delete, get, post},
};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

use crate::{
    api::{devices, health, pairing as pairing_api, websocket},
    config::Config,
    error::CoreError,
    mode::ModeManager,
    pairing::TokenStore,
    state::{AppState, ClientRegistry, RequestCache},
    system::create_system_backend,
};

/// 検証済み設定から、共有状態とOSバックエンドを初期化する。
///
/// 起動時の実表示モードを照合できない場合は、不正な初期状態で配信を始めず失敗を返す。
pub async fn build_state(config: Config) -> Result<AppState, CoreError> {
    let backend = create_system_backend(&config).await?;
    let mode_manager = Arc::new(ModeManager::with_timeout(backend, config.command_timeout).await?);
    let token_store = Arc::new(TokenStore::load(config.token_store.clone()).await?);
    Ok(AppState {
        config: Arc::new(config),
        clients: ClientRegistry::default(),
        mode_manager,
        token_store,
        requests: RequestCache::default(),
        pairing_failures: Default::default(),
    })
}

/// CoreのAPIとTV静的ファイルを同一オリジンで配信するルーターを構築する。
pub fn build_router(state: AppState) -> Router {
    let index = state.config.static_dir.join("index.html");
    let static_files =
        ServeDir::new(state.config.static_dir.clone()).not_found_service(ServeFile::new(index));

    Router::new()
        .route("/health/live", get(health::live))
        .route("/health/ready", get(health::ready))
        .route("/api/v1/pair", post(pairing_api::pair))
        .route("/api/v1/devices", get(devices::list))
        .route("/api/v1/devices/{device_id}", delete(devices::revoke))
        .route("/ws", any(websocket::upgrade))
        .fallback_service(static_files)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
