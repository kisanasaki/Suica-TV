# WebSocket API 詳細設計書

## 1. 目的と範囲

本書はSuica Remote、Suica Core、Suica TV Home間の通信契約を定義する。メッセージ形式、接続、認証、役割、順序、応答、タイムアウト、Keepalive、切断コードおよび互換性を対象とする。

## 2. エンドポイント

```text
ws://{host}:3030/ws?role={remote|tv}&protocolVersion=1
```

| パラメーター | 必須 | 値 |
|---|---|---|
| `role` | 必須 | `remote`または`tv` |
| `protocolVersion` | 必須 | v0.1では`1` |

本番でWSSを導入する場合もパスとメッセージ契約は変更しない。

## 3. 認証

### 3.1 Remote

HTTP Upgrade要求へ次のヘッダーを付与する。

```http
Authorization: Bearer <pairing-token>
```

トークンがない、無効、失効済みの場合はUpgradeせずHTTP 401を返す。権限はリモコン操作だけに限定し、シェル実行や任意URL起動は許可しない。

### 3.2 TV

TVロールは接続元がループバックアドレスである場合だけ許可する。将来別ホストで表示する場合は専用トークン認証へ拡張する。

## 4. 接続確立

Upgrade成功後、Coreは最初のメッセージとして`server.hello`、続けて`system.state`を送信する。

```json
{
  "type": "server.hello",
  "protocolVersion": 1,
  "serverVersion": "0.1.0",
  "connectionId": "2c13bd30-2ad4-4c98-b79a-b9a4e3e41ee5",
  "role": "remote"
}
```

`protocolVersion`が一致しない場合、Coreは`unsupported_protocol`エラー送信後に1002で切断する。

## 5. 共通規則

- UTF-8のJSONテキストフレームだけを使用する。
- バイナリフレームは1003で拒否する。
- 1メッセージの最大サイズは64KiBとする。
- JSONのプロパティは`camelCase`とする。
- 未定義プロパティはv0.1では無視する。
- 必須プロパティ欠落、型不一致、未知のenum値はエラーとする。
- クライアント要求にはUUID v4の`requestId`を付与する。
- サーバーイベントには`requestId`がない場合がある。
- 同一接続内の送信順序を維持する。

## 6. ClientからCoreへのメッセージ

### 6.1 remote.command

```json
{
  "type": "remote.command",
  "requestId": "550e8400-e29b-41d4-a716-446655440000",
  "action": "navigation.up"
}
```

#### 操作一覧

| action | params | Remote送信 | TV送信 |
|---|---|---:|---:|
| `navigation.up` | なし | 可 | 不可 |
| `navigation.down` | なし | 可 | 不可 |
| `navigation.left` | なし | 可 | 不可 |
| `navigation.right` | なし | 可 | 不可 |
| `navigation.select` | なし | 可 | 不可 |
| `navigation.back` | なし | 可 | 不可 |
| `navigation.home` | なし | 可 | 不可 |
| `system.switch_mode` | `mode` | 可 | PCへの切り替えのみ可 |
| `pointer.scroll` | `dx`, `dy` | 可 | 不可 |
| `input.text` | `text` | 可 | 不可 |
| `input.delete_backward` | なし | 可 | 不可 |
| `input.submit` | なし | 可 | 不可 |

`system.switch_mode`:

```json
{
  "type": "remote.command",
  "requestId": "550e8400-e29b-41d4-a716-446655440000",
  "action": "system.switch_mode",
  "params": {
    "mode": "pc"
  }
}
```

TVロールからの`system.switch_mode`は、ループバック接続かつ`params.mode`が`pc`の場合だけ許可する。TVロールからのTV切り替えおよびその他のコマンド送信は拒否する。

`input.text`の`text`は1〜200文字とし、制御文字を拒否する。検索語などの秘密でない入力も
ログへ出力しない。ReactのTV接続がないTVモードでは、Coreが`navigation.*`をChromiumの
キーボード操作へ変換する。`input.*`は常にCoreからChromiumへ直接入力する。

`navigation.back`はReact接続中はTVアプリ内の前画面へ転送する。外部ページ表示中などTV接続が
ない場合はChromiumの履歴戻り（Alt+Left）へ変換する。戻り先がない場合は何も変更されず、
`navigation.home`によるホーム復帰とは区別する。

`pointer.scroll`の`dx`と`dy`は相対スクロール量で、各軸-1200〜1200とする。両方が0の要求は
拒否する。iOSは連続操作を75ミリ秒ごとに集約し、Coreは入力バックエンドに応じて安全な
ホイール操作または方向キー操作へ変換する。

## 7. CoreからClientへのメッセージ

### 7.1 remote.command

Coreは検証済みのナビゲーション操作をTVクライアントへ転送する。元の`requestId`を維持する。

```json
{
  "type": "remote.command",
  "requestId": "550e8400-e29b-41d4-a716-446655440000",
  "action": "navigation.up"
}
```

### 7.2 system.state

```json
{
  "type": "system.state",
  "mode": "tv",
  "transitioning": false,
  "changedAt": "2026-09-21T10:00:00Z"
}
```

| フィールド | 型 | 説明 |
|---|---|---|
| `mode` | `tv`または`pc` | 最後に確定したモード |
| `transitioning` | boolean | 切り替え処理中か |
| `targetMode` | mode、省略可 | 切り替え先 |
| `changedAt` | RFC 3339 | 確定状態の更新時刻 |

切り替え開始時は`transitioning: true`、完了または失敗後は`false`を通知する。失敗後の`mode`は直前の確定状態とする。

### 7.3 command.result

成功:

```json
{
  "type": "command.result",
  "requestId": "550e8400-e29b-41d4-a716-446655440000",
  "ok": true
}
```

失敗:

```json
{
  "type": "command.result",
  "requestId": "550e8400-e29b-41d4-a716-446655440000",
  "ok": false,
  "error": {
    "code": "mode_switch_failed",
    "message": "表示モードを切り替えられませんでした。",
    "retryable": true
  }
}
```

### 7.4 error

要求へ紐付けられない接続単位の問題に使用する。

```json
{
  "type": "error",
  "error": {
    "code": "invalid_message",
    "message": "メッセージ形式が不正です。",
    "retryable": false
  }
}
```

### 7.5 server.shutdown

```json
{
  "type": "server.shutdown",
  "retryAfterSeconds": 3
}
```

クライアントは切断後、指定秒数以降に再接続する。

## 8. エラーコード

| code | HTTP / WS | retryable | 意味 |
|---|---|---:|---|
| `unauthorized` | HTTP 401 | false | トークンが無効 |
| `forbidden_role` | HTTP 403 | false | 接続元に対してroleが不正 |
| `unsupported_protocol` | WS 1002 | false | プロトコル非対応 |
| `invalid_message` | message | false | JSONまたは必須項目が不正 |
| `invalid_command` | result | false | 未定義の操作 |
| `invalid_state` | result | true | 現在状態では実行不可 |
| `busy` | result | true | モード切り替え実行中 |
| `tv_client_unavailable` | result | true | TVクライアント未接続 |
| `mode_switch_failed` | result | true | OS操作に失敗 |
| `timeout` | result | true | 処理時間超過 |
| `internal_error` | result | true | Core内部エラー |

## 9. 応答規則

| 操作 | command.result | system.state |
|---|---|---|
| Navigation | TVへのキュー投入後に成功 | なし |
| Home | ホーム表示確認後に成功 | TVへの遷移があれば送信 |
| SwitchMode | 切り替え完了後に成功 | 開始時と完了時に送信 |

クライアントは`command.result`と`system.state`の到着順に依存しない。`requestId`は処理結果の対応、`system.state`は最終的な表示状態の同期に使用する。

## 10. タイムアウトと冪等性

- ナビゲーション操作のCore内処理タイムアウトは2秒とする。
- モード切り替えのタイムアウトは10秒とする。
- クライアントの応答待ちタイムアウトは5秒とする。ただしモード切り替えは12秒とする。
- 同一`requestId`の再受信時は60秒以内なら保存済み結果を返す。
- 異なる`requestId`の同一操作は別要求として扱う。

## 11. Keepalive

- Coreは20秒ごとにWebSocket Pingを送る。
- クライアントはプロトコル標準のPongを返す。
- 10秒以内にPongがなければ1001で切断する。
- アプリケーションJSONで独自Pingを実装しない。

## 12. 切断コード

| Code | 用途 |
|---:|---|
| 1000 | クライアントによる正常終了 |
| 1001 | サーバー終了、Keepalive失敗 |
| 1002 | プロトコルバージョン不一致 |
| 1003 | バイナリフレーム受信 |
| 1008 | 認証後のポリシー違反 |
| 1009 | 64KiBを超えるメッセージ |
| 1011 | 回復不能なサーバー内部エラー |

## 13. シーケンス

通常操作、モード切り替え、再接続のシーケンスは[draw.io](diagrams/websocket-api.drawio)に定義する。

## 14. セキュリティ

- LAN内運用でもトークンを必須とする。
- トークンをURL、ログ、エラー本文へ含めない。
- OriginがアプリまたはローカルTV以外の場合は拒否できる設定を用意する。
- 1接続あたり毎秒20コマンド、バースト40を上限とする。
- 認証失敗は接続元IPごとに1分10回までとする。
- `params`からコマンド名、URL、ファイルパスを直接生成しない。

## 15. 互換性方針

- `protocolVersion`のメジャー値が同じ場合、未知の追加プロパティは無視する。
- 必須プロパティの削除、型変更、enum値の意味変更はメジャー更新とする。
- 新しい任意メッセージは古いクライアントが無視できる設計にする。
- `serverVersion`は診断用であり、互換性判定には使用しない。

## 16. 契約テスト

| ID | 確認内容 |
|---|---|
| API-C-01 | 全メッセージ例をRust、Swift、TypeScriptで解析できる。 |
| API-C-02 | 未知プロパティを無視できる。 |
| API-C-03 | 未知actionを拒否する。 |
| API-C-04 | UUIDでないrequestIdを拒否する。 |
| API-C-05 | 64KiB超過を1009で切断する。 |
| API-C-06 | 無効トークンでUpgradeしない。 |
| API-C-07 | TV roleのLAN接続を拒否する。 |
| API-C-08 | 重複requestIdで処理を再実行しない。 |
| API-C-09 | Ping不応答クライアントを切断する。 |
| API-C-10 | プロトコル不一致を1002で切断する。 |

## 17. 受入基準

- 3実装が同じJSON契約を共有できる。
- 再接続や重複送信でシステム操作が二重実行されない。
- 認証情報がURLとログへ露出しない。
- モード切り替え失敗後も全クライアントの状態が一致する。
