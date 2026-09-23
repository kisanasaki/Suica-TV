use crate::{
    error::CoreError,
    mode::DisplayMode,
    remote::{ErrorBody, NavigationAction, RemoteAction, RemoteCommand, ServerMessage},
    state::{AppState, ClientRole, CommandRateLimiter, RequestDecision, is_loopback},
    system::BrowserKey,
};
use axum::{
    extract::{
        ConnectInfo, Query, State,
        ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade},
    },
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};
use tokio::{sync::mpsc, time::interval};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsQuery {
    role: String,
    protocol_version: u8,
}

pub async fn upgrade(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Query(query): Query<WsQuery>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Response {
    let role = match query.role.as_str() {
        "remote" => ClientRole::Remote,
        "tv" => ClientRole::Tv,
        _ => return CoreError::ForbiddenRole.into_response(),
    };
    if query.protocol_version != 1 {
        return CoreError::UnsupportedProtocol.into_response();
    }
    if role == ClientRole::Tv && !is_loopback(addr.ip()) {
        return CoreError::ForbiddenRole.into_response();
    }
    if role == ClientRole::Remote {
        let token = headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "));
        match token {
            Some(token) if state.token_store.verify(token).await => {}
            _ => return CoreError::Unauthorized.into_response(),
        }
    }
    ws.max_message_size(64 * 1024)
        .max_frame_size(64 * 1024)
        .on_upgrade(move |socket| handle_socket(socket, state, role))
        .into_response()
}

async fn handle_socket(mut socket: WebSocket, state: AppState, role: ClientRole) {
    let (connection_id, mut outbound) = match state.clients.register(role).await {
        Ok(v) => v,
        Err(e) => {
            let _ = socket
                .send(Message::Text(
                    ServerMessage::Error {
                        error: ErrorBody::from_error(&e),
                    }
                    .text()
                    .into(),
                ))
                .await;
            let _ = socket.close().await;
            return;
        }
    };
    let hello = ServerMessage::Hello {
        protocol_version: 1,
        server_version: env!("CARGO_PKG_VERSION").into(),
        connection_id,
        role: role.as_str().into(),
    };
    let snapshot = state.mode_manager.snapshot().await;
    let system = ServerMessage::SystemState {
        mode: snapshot.mode,
        transitioning: snapshot.transitioning,
        target_mode: snapshot.target,
        changed_at: snapshot.changed_at,
    };
    if socket
        .send(Message::Text(hello.text().into()))
        .await
        .is_err()
        || socket
            .send(Message::Text(system.text().into()))
            .await
            .is_err()
    {
        state.clients.remove(connection_id).await;
        return;
    }
    let (mut sender, mut receiver) = socket.split();
    let mut ping = interval(Duration::from_secs(20));
    ping.tick().await;
    let mut last_pong = Instant::now();
    let mut limiter = CommandRateLimiter::default();
    let (tx, mut local_rx) = mpsc::channel::<ServerMessage>(32);
    let connection = async {
        loop {
            tokio::select! {
                msg=outbound.recv()=>{match msg {
                    Some(msg) => if sender.send(Message::Text(msg.text().into())).await.is_err(){break},
                    None => {
                        let _=sender.send(Message::Close(Some(CloseFrame{code:1001,reason:"replaced by a new connection".into()}))).await;
                        break
                    },
                }},
                Some(msg)=local_rx.recv()=>{if sender.send(Message::Text(msg.text().into())).await.is_err(){break}},
                incoming=receiver.next()=>{match incoming{
                    Some(Ok(Message::Text(text)))=>{if !limiter.allow(){let e=CoreError::Busy;let _=tx.try_send(ServerMessage::Error{error:ErrorBody::from_error(&e)});continue}match RemoteCommand::parse(&text){Ok(command)=>{let st=state.clone();let out=tx.clone();tokio::spawn(async move{process_command(st,role,command,out).await;});},Err(e)=>{let _=tx.try_send(ServerMessage::Error{error:ErrorBody::from_error(&e)});}}},
                    Some(Ok(Message::Pong(_)))=>last_pong=Instant::now(),
                    Some(Ok(Message::Ping(data)))=>{if sender.send(Message::Pong(data)).await.is_err(){break}},
                    Some(Ok(Message::Binary(_)))=>{let _=sender.send(Message::Close(Some(CloseFrame{code:1003,reason:"text frames only".into()}))).await;break},
                    Some(Ok(Message::Close(_)))|None|Some(Err(_))=>break,
                }},
            _=ping.tick()=>{
                if last_pong.elapsed()>Duration::from_secs(30){
                    let _=sender.send(Message::Close(Some(CloseFrame{code:1001,reason:"keepalive timeout".into()}))).await;
                    break
                }
                if sender.send(Message::Ping(Vec::new().into())).await.is_err(){break}
            },
            }
        }
    };
    connection.await;
    state.clients.remove(connection_id).await;
}

async fn process_command(
    state: AppState,
    role: ClientRole,
    command: RemoteCommand,
    out: mpsc::Sender<ServerMessage>,
) {
    match state.requests.begin(command.request_id).await {
        RequestDecision::Cached(cached) => {
            let _ = out.send(cached).await;
            return;
        }
        RequestDecision::Execute => {}
    }
    let result = dispatch(&state, role, &command).await;
    let message = match result {
        Ok(()) => ServerMessage::CommandResult {
            request_id: command.request_id,
            ok: true,
            error: None,
        },
        Err(e) => {
            tracing::warn!(request_id=%command.request_id,error=%e,"command failed");
            ServerMessage::CommandResult {
                request_id: command.request_id,
                ok: false,
                error: Some(ErrorBody::from_error(&e)),
            }
        }
    };
    state
        .requests
        .finish(command.request_id, message.clone())
        .await;
    let _ = out.send(message).await;
}

async fn dispatch(
    state: &AppState,
    role: ClientRole,
    command: &RemoteCommand,
) -> Result<(), CoreError> {
    match &command.action {
        RemoteAction::Navigation(action) => {
            if role != ClientRole::Remote {
                return Err(CoreError::ForbiddenRole);
            }
            if *action == NavigationAction::Home {
                state
                    .mode_manager
                    .ensure_tv_home(command.request_id)
                    .await?;
                broadcast_mode(state).await;
                return Ok(());
            } else if state.mode_manager.snapshot().await.mode != DisplayMode::Tv {
                return Err(CoreError::InvalidState);
            }
            let forwarded = state
                .clients
                .send_tv(ServerMessage::RemoteCommand {
                    request_id: command.request_id,
                    action: action.wire_name().into(),
                })
                .await;
            match forwarded {
                Ok(()) => Ok(()),
                Err(CoreError::TvClientUnavailable) => {
                    state
                        .mode_manager
                        .send_browser_key(browser_key(*action))
                        .await
                }
                Err(error) => Err(error),
            }
        }
        RemoteAction::SwitchMode(target) => {
            if role == ClientRole::Tv && *target != DisplayMode::Pc {
                return Err(CoreError::ForbiddenRole);
            }
            let before = state.mode_manager.snapshot().await;
            state
                .clients
                .broadcast(ServerMessage::SystemState {
                    mode: before.mode,
                    transitioning: true,
                    target_mode: Some(*target),
                    changed_at: before.changed_at,
                })
                .await;
            let result = state.mode_manager.switch(*target, command.request_id).await;
            broadcast_mode(state).await;
            result.map(|_| ())
        }
        RemoteAction::Scroll { dx, dy } => {
            ensure_remote_tv_mode(state, role).await?;
            state.mode_manager.scroll_browser(*dx, *dy).await
        }
        RemoteAction::InputText(text) => {
            ensure_remote_tv_mode(state, role).await?;
            state.mode_manager.type_browser_text(text).await
        }
        RemoteAction::DeleteBackward => {
            ensure_remote_tv_mode(state, role).await?;
            state
                .mode_manager
                .send_browser_key(BrowserKey::Backspace)
                .await
        }
        RemoteAction::SubmitText => {
            ensure_remote_tv_mode(state, role).await?;
            state.mode_manager.send_browser_key(BrowserKey::Enter).await
        }
    }
}

fn browser_key(action: NavigationAction) -> BrowserKey {
    match action {
        NavigationAction::Up => BrowserKey::Up,
        NavigationAction::Down => BrowserKey::Down,
        NavigationAction::Left => BrowserKey::Left,
        NavigationAction::Right => BrowserKey::Right,
        NavigationAction::Select => BrowserKey::Enter,
        NavigationAction::Back => BrowserKey::Escape,
        NavigationAction::Home => unreachable!("home is handled before browser input"),
    }
}

async fn ensure_remote_tv_mode(state: &AppState, role: ClientRole) -> Result<(), CoreError> {
    if role != ClientRole::Remote {
        return Err(CoreError::ForbiddenRole);
    }
    if state.mode_manager.snapshot().await.mode != DisplayMode::Tv {
        return Err(CoreError::InvalidState);
    }
    Ok(())
}

pub async fn broadcast_mode(state: &AppState) {
    let s = state.mode_manager.snapshot().await;
    state
        .clients
        .broadcast(ServerMessage::SystemState {
            mode: s.mode,
            transitioning: s.transitioning,
            target_mode: s.target,
            changed_at: s.changed_at,
        })
        .await;
}

pub async fn notify_shutdown(state: &AppState) {
    state
        .clients
        .broadcast(ServerMessage::Shutdown {
            retry_after_seconds: 3,
        })
        .await;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn query_uses_camel_case() {
        let q: WsQuery = serde_json::from_str(r#"{"role":"tv","protocolVersion":1}"#).unwrap();
        assert_eq!(q.protocol_version, 1);
    }
}
