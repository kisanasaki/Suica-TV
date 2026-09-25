//! Remote API v1で送受信するコマンドとサーバーメッセージを公開する。
//!
//! Wire形式の解析と生成を集約し、APIハンドラーからserde表現の詳細を分離する。

pub mod command;
pub mod message;

pub use command::*;
pub use message::*;
