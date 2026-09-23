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
