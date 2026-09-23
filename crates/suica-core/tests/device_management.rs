use axum::{
    body::{Body, to_bytes},
    extract::ConnectInfo,
    http::{Request, StatusCode},
};
use serde_json::Value;
use std::net::SocketAddr;
use suica_core::{
    build_router, build_state,
    config::{Config, SystemBackendKind},
};
use tower::ServiceExt;

async fn test_state() -> (suica_core::state::AppState, tempfile::TempDir) {
    let temp = tempfile::tempdir().unwrap();
    let static_dir = temp.path().join("dist");
    tokio::fs::create_dir_all(&static_dir).await.unwrap();
    tokio::fs::write(static_dir.join("index.html"), "<h1>Suica TV</h1>")
        .await
        .unwrap();
    let state = build_state(Config {
        bind_address: "127.0.0.1".parse().unwrap(),
        port: 0,
        static_dir,
        token_store: temp.path().join("tokens.json"),
        pairing_code: "123456".into(),
        system_backend: SystemBackendKind::Simulated,
        ..Config::default()
    })
    .await
    .unwrap();
    (state, temp)
}

fn loopback_request(method: &str, uri: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .extension(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 4567))))
        .body(Body::empty())
        .unwrap()
}

#[tokio::test]
async fn loopback_can_list_and_revoke_one_device() {
    let (state, _temp) = test_state().await;
    let (first_id, first_token) = state.token_store.issue("first".into()).await.unwrap();
    let (_, second_token) = state.token_store.issue("second".into()).await.unwrap();
    let app = build_router(state.clone());

    let response = app
        .clone()
        .oneshot(loopback_request("GET", "/api/v1/devices"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(body["devices"].as_array().unwrap().len(), 2);

    let response = app
        .clone()
        .oneshot(loopback_request(
            "DELETE",
            &format!("/api/v1/devices/{first_id}"),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert!(!state.token_store.verify(&first_token).await);
    assert!(state.token_store.verify(&second_token).await);

    let response = app
        .oneshot(loopback_request(
            "DELETE",
            &format!("/api/v1/devices/{first_id}"),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn non_loopback_device_management_is_forbidden() {
    let (state, _temp) = test_state().await;
    let request = Request::builder()
        .uri("/api/v1/devices")
        .extension(ConnectInfo(SocketAddr::from(([192, 0, 2, 10], 4567))))
        .body(Body::empty())
        .unwrap();
    let response = build_router(state).oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
