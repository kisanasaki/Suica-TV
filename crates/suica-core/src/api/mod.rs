//! Suica Coreが公開するHTTP/WebSocketハンドラーをまとめる。
//!
//! ルーティング自体はcrateルートで行い、各サブモジュールが認証と入力検証を担当する。

pub mod devices;
pub mod health;
pub mod pairing;
pub mod websocket;
