use std::{env, path::PathBuf, time::Duration};
use suica_core::{api::websocket, build_router, build_state, config::Config, error::CoreError};
use tokio::signal;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), CoreError> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("suica_core=info,tower_http=info")),
        )
        .json()
        .init();
    let args: Vec<String> = env::args().collect();
    let path = args
        .windows(2)
        .find(|w| w[0] == "--config")
        .map(|w| PathBuf::from(&w[1]));
    let config = Config::load(path.as_deref())?;
    let addr = (config.bind_address, config.port);
    let state = build_state(config).await?;
    let app = build_router(state.clone());
    let listener = tokio::net::TcpListener::bind(addr).await?;
    #[cfg(target_os = "linux")]
    {
        let manager = state.mode_manager.clone();
        tokio::spawn(async move {
            if let Err(e) = manager.ensure_tv_home(uuid::Uuid::new_v4()).await {
                tracing::error!(error=%e,"failed to start TV mode")
            }
        });
    }
    tracing::info!(address=%listener.local_addr()?,"Suica Core started");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown(state))
    .await
    .map_err(|e| CoreError::Internal(e.to_string()))
}
async fn shutdown(state: suica_core::state::AppState) {
    let ctrl_c = async { signal::ctrl_c().await.expect("Ctrl+C handler") };
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("signal handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {_ = ctrl_c=>{},_=terminate=>{}}
    websocket::notify_shutdown(&state).await;
    tokio::time::sleep(Duration::from_secs(1)).await;
}
