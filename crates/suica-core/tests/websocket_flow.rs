//! Remote、Core、TV間の主要なWebSocket操作を一連の接続で検証する。
//!
//! Reactへの転送、外部ページ用OS入力、モード切替、ホーム復帰、
//! TV再接続をまとめて確認し、層をまたぐ回帰を検出する。

use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use suica_core::{
    build_router, build_state,
    config::{Config, SystemBackendKind},
};
use tokio::time::{Duration, timeout};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Message, client::IntoClientRequest, http::HeaderValue},
};
use uuid::Uuid;

async fn receive_json<S>(socket: &mut S) -> Value
where
    S: StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
{
    loop {
        let message = timeout(Duration::from_secs(2), socket.next())
            .await
            .expect("websocket response timed out")
            .expect("websocket closed")
            .expect("websocket error");
        if let Message::Text(text) = message {
            return serde_json::from_str(&text).expect("valid JSON message");
        }
    }
}

#[tokio::test]
async fn remote_commands_reach_tv_and_tv_can_switch_to_pc() {
    let temp = tempfile::tempdir().unwrap();
    let static_dir = temp.path().join("dist");
    tokio::fs::create_dir_all(&static_dir).await.unwrap();
    tokio::fs::write(static_dir.join("index.html"), "<h1>Suica TV</h1>")
        .await
        .unwrap();

    let config = Config {
        bind_address: "127.0.0.1".parse().unwrap(),
        port: 0,
        static_dir,
        token_store: temp.path().join("tokens.json"),
        pairing_code: "123456".into(),
        system_backend: SystemBackendKind::Simulated,
        ..Config::default()
    };
    let state = build_state(config).await.unwrap();
    let (_, token) = state.token_store.issue("test remote".into()).await.unwrap();
    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });

    let tv_url = format!("ws://{address}/ws?role=tv&protocolVersion=1");
    let (mut tv, _) = connect_async(&tv_url).await.unwrap();
    assert_eq!(receive_json(&mut tv).await["type"], "server.hello");
    assert_eq!(receive_json(&mut tv).await["mode"], "tv");

    let remote_url = format!("ws://{address}/ws?role=remote&protocolVersion=1");
    let mut request = remote_url.into_client_request().unwrap();
    request.headers_mut().insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
    );
    let (mut remote, _) = connect_async(request).await.unwrap();
    assert_eq!(receive_json(&mut remote).await["type"], "server.hello");
    receive_json(&mut remote).await;

    let navigation_id = Uuid::new_v4();
    remote
        .send(Message::Text(
            json!({
                "type": "remote.command",
                "requestId": navigation_id,
                "action": "navigation.right"
            })
            .to_string()
            .into(),
        ))
        .await
        .unwrap();
    let forwarded = receive_json(&mut tv).await;
    assert_eq!(forwarded["type"], "remote.command");
    assert_eq!(forwarded["action"], "navigation.right");
    let result = receive_json(&mut remote).await;
    assert_eq!(result["requestId"], navigation_id.to_string());
    assert_eq!(result["ok"], true);

    let media_on_home_id = Uuid::new_v4();
    remote
        .send(Message::Text(
            json!({
                "type": "remote.command",
                "requestId": media_on_home_id,
                "action": "media.play_pause"
            })
            .to_string()
            .into(),
        ))
        .await
        .unwrap();
    let media_on_home = receive_json(&mut remote).await;
    assert_eq!(media_on_home["requestId"], media_on_home_id.to_string());
    assert_eq!(media_on_home["ok"], false);
    assert_eq!(media_on_home["error"]["code"], "invalid_state");

    drop(tv);
    tokio::time::sleep(Duration::from_millis(25)).await;
    for action in [
        "navigation.down",
        "pointer.scroll",
        "pointer.move",
        "pointer.click",
        "input.text",
        "input.delete_backward",
        "input.submit",
        "media.play_pause",
        "media.seek_backward",
        "media.seek_forward",
        "media.fullscreen_toggle",
    ] {
        let request_id = Uuid::new_v4();
        let params = match action {
            "input.text" => Some(json!({"text": "日本語🍉"})),
            "pointer.scroll" => Some(json!({"dx": 0, "dy": 240})),
            "pointer.move" => Some(json!({"dx": 20, "dy": -10})),
            _ => None,
        };
        let mut command = json!({
            "type": "remote.command",
            "requestId": request_id,
            "action": action
        });
        if let Some(params) = params {
            command["params"] = params;
        }
        remote
            .send(Message::Text(command.to_string().into()))
            .await
            .unwrap();
        let result = receive_json(&mut remote).await;
        assert_eq!(result["requestId"], request_id.to_string());
        assert_eq!(result["ok"], true);
    }

    let switch_id = Uuid::new_v4();
    let (mut tv, _) = connect_async(&tv_url).await.unwrap();
    receive_json(&mut tv).await;
    receive_json(&mut tv).await;
    tv.send(Message::Text(
        json!({
            "type": "remote.command",
            "requestId": switch_id,
            "action": "system.switch_mode",
            "params": {"mode": "pc"}
        })
        .to_string()
        .into(),
    ))
    .await
    .unwrap();
    let mut saw_pc = false;
    let mut saw_result = false;
    for _ in 0..3 {
        let message = receive_json(&mut tv).await;
        saw_pc |= message["type"] == "system.state" && message["mode"] == "pc";
        saw_result |= message["type"] == "command.result" && message["ok"] == true;
    }
    assert!(saw_pc && saw_result);

    let forbidden_id = Uuid::new_v4();
    let forbidden = json!({
        "type": "remote.command",
        "requestId": forbidden_id,
        "action": "system.switch_mode",
        "params": {"mode": "tv"}
    })
    .to_string();
    tv.send(Message::Text(forbidden.clone().into()))
        .await
        .unwrap();
    let first = receive_json(&mut tv).await;
    assert_eq!(first["error"]["code"], "forbidden_role");

    tv.send(Message::Text(forbidden.into())).await.unwrap();
    let duplicate = receive_json(&mut tv).await;
    assert_eq!(duplicate["requestId"], forbidden_id.to_string());
    assert_eq!(duplicate["error"]["code"], "forbidden_role");

    let home_id = Uuid::new_v4();
    remote
        .send(Message::Text(
            json!({
                "type": "remote.command",
                "requestId": home_id,
                "action": "navigation.home"
            })
            .to_string()
            .into(),
        ))
        .await
        .unwrap();
    let mut saw_home_state = false;
    let mut saw_home_result = false;
    for _ in 0..6 {
        let message = receive_json(&mut remote).await;
        saw_home_state |= message["type"] == "system.state"
            && message["mode"] == "tv"
            && message["transitioning"] == false;
        saw_home_result |= message["type"] == "command.result"
            && message["requestId"] == home_id.to_string()
            && message["ok"] == true;
        if saw_home_state && saw_home_result {
            break;
        }
    }
    assert!(saw_home_state && saw_home_result);

    let (mut replacement_tv, _) = connect_async(&tv_url).await.unwrap();
    assert_eq!(
        receive_json(&mut replacement_tv).await["type"],
        "server.hello"
    );
    assert_eq!(receive_json(&mut replacement_tv).await["mode"], "tv");
    let select_id = Uuid::new_v4();
    remote
        .send(Message::Text(
            json!({
                "type": "remote.command",
                "requestId": select_id,
                "action": "navigation.select"
            })
            .to_string()
            .into(),
        ))
        .await
        .unwrap();
    let select = receive_json(&mut replacement_tv).await;
    assert_eq!(select["type"], "remote.command");
    assert_eq!(select["action"], "navigation.select");
    let result = receive_json(&mut remote).await;
    assert_eq!(result["requestId"], select_id.to_string());
    assert_eq!(result["ok"], true);

    server.abort();
}
