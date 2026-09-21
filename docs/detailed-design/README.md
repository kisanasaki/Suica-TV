# Suica TV v0.1 詳細設計書

本ディレクトリは、[基本設計書](../design.md)を実装単位へ具体化した詳細設計書である。仕様が競合する場合は基本設計書を上位文書とし、詳細設計の修正が必要な場合は両文書の整合性を保つ。

## 文書一覧

| 文書 | 対象 | 関連図 |
|---|---|---|
| [01 Suica TV Home](01-suica-tv-home.md) | React / TypeScriptテレビUI | [tv-home.drawio](diagrams/tv-home.drawio) |
| [02 Suica Remote](02-suica-remote.md) | Swift / SwiftUI iPhoneアプリ | [suica-remote.drawio](diagrams/suica-remote.drawio) |
| [03 Suica Core](03-suica-core.md) | Rust / Tokio / Axumサービス | [suica-core.drawio](diagrams/suica-core.drawio) |
| [04 Chromium OS連携](04-system-integration.md) | Chromium、systemd、起動スクリプト | [system-integration.drawio](diagrams/system-integration.drawio) |
| [05 WebSocket API](05-websocket-api.md) | コンポーネント間の通信契約 | [websocket-api.drawio](diagrams/websocket-api.drawio) |

## 共通設計規約

- 日付・時刻は内部ではUTCで保持し、表示時のみ端末のローカル時刻へ変換する。
- 識別子はUUID v4を使用する。
- JSONのプロパティ名は`camelCase`、Rustの内部名は`snake_case`とする。
- モード値は`tv`と`pc`に限定する。
- 操作名は`navigation.*`と`system.*`の名前空間に分ける。
- 外部入力は利用前に必ず検証し、未知の値を黙って受け入れない。
- ペアリングトークン、認証ヘッダー、個人情報はログへ出力しない。
- ユーザー向けエラー文と診断用ログを分離する。
- v0.1では後方互換性のため、通信プロトコルのバージョンを`1`として扱う。

## 要件と文書の対応

| 基本要件 | 主担当文書 | 関連文書 |
|---|---|---|
| F-001 React製ホーム画面 | 01 | 03、05 |
| F-002 iPhone専用リモコン | 02 | 03、05 |
| F-003 表示モード切り替え | 03、04 | 02、05 |
| 自動起動・自動復旧 | 04 | 03 |
| ペアリングとコマンド制限 | 03、05 | 02 |

## 実装時の決定事項

詳細設計で次の点を基本設計より具体化した。

1. WebSocketエンドポイントは`/ws`とし、接続クライアントの役割を`role`で識別する。
2. iPhoneクライアントはBearerトークンで認証し、TVクライアントはループバック接続に限定する。
3. Suica CoreをReact静的ファイルの配信元とし、本番URLを`http://127.0.0.1:3030/`へ統一する。
4. ホーム復帰は、TVモードを保証した後にホームURLへChromiumを再表示する処理とする。
5. モード切り替えは1件ずつ実行し、実行中の追加要求には`busy`エラーを返す。
