# Suica Core Remote API v1

## HTTP

### `GET /health/live`

プロセスが応答できる場合は`200 OK`を返す。

### `GET /health/ready`

Reactの`index.html`を配信できる場合は`200 OK`、準備できていない場合は`503 Service Unavailable`を返す。

### `POST /api/v1/pair`

```json
{
  "code": "123456",
  "deviceName": "My iPhone"
}
```

成功時は`201 Created`を返す。トークンの平文はこの応答で一度だけ提供される。

```json
{
  "deviceId": "550e8400-e29b-41d4-a716-446655440000",
  "token": "43文字のBase64URLトークン"
}
```

コード認証の失敗は接続元IPごとに1分10回まで。`deviceName`は空文字を許可せず、最大64文字とする。

## WebSocket

接続先:

```text
GET /ws?role=remote|tv&protocolVersion=1
```

- `remote`: `Authorization: Bearer <token>`が必須。同時4接続まで。
- `tv`: ループバック接続だけを許可。同時1接続まで。
- メッセージはUTF-8 JSONテキスト、最大64KiB。
- 20秒ごとにPingを送信し、応答がない接続を切断する。

接続直後、`server.hello`と現在の`system.state`を順に受信する。

```json
{
  "type": "server.hello",
  "protocolVersion": 1,
  "serverVersion": "0.1.0",
  "connectionId": "UUID",
  "role": "remote"
}
```

```json
{
  "type": "system.state",
  "mode": "tv",
  "transitioning": false,
  "changedAt": "2026-09-21T00:00:00Z"
}
```

## コマンド

全コマンドにクライアント生成のUUID `requestId`を指定する。同じIDの結果は60秒保持され、再送時にシステム操作を再実行せず同じ結果を返す。

```json
{
  "type": "remote.command",
  "requestId": "550e8400-e29b-41d4-a716-446655440000",
  "action": "navigation.up"
}
```

操作名:

- `navigation.up`
- `navigation.down`
- `navigation.left`
- `navigation.right`
- `navigation.select`
- `navigation.back`
- `navigation.home`
- `system.switch_mode`

モード切り替え:

```json
{
  "type": "remote.command",
  "requestId": "550e8400-e29b-41d4-a716-446655440000",
  "action": "system.switch_mode",
  "params": { "mode": "pc" }
}
```

TVロールは`system.switch_mode`の`pc`だけを送信できる。Remoteロールの`navigation.*`はTVロールへ中継される。

成功応答:

```json
{
  "type": "command.result",
  "requestId": "550e8400-e29b-41d4-a716-446655440000",
  "ok": true
}
```

失敗応答:

```json
{
  "type": "command.result",
  "requestId": "550e8400-e29b-41d4-a716-446655440000",
  "ok": false,
  "error": {
    "code": "forbidden_role",
    "message": "この接続では操作できません。",
    "retryable": false
  }
}
```

エラーコードは次に限定する。

- `unauthorized`
- `forbidden_role`
- `unsupported_protocol`
- `invalid_message`
- `invalid_command`
- `invalid_state`
- `busy`
- `tv_client_unavailable`
- `mode_switch_failed`
- `timeout`
- `internal_error`

Core終了時:

```json
{
  "type": "server.shutdown",
  "retryAfterSeconds": 3
}
```
