# Suica Core 詳細設計書

## 1. 目的と範囲

Suica Coreは、Raspberry Pi上で動作する制御サービスである。Suica RemoteとSuica TV Homeの接続を管理し、許可済みの操作を振り分け、表示モードとChromiumプロセスを制御する。

本書はクレート構造、共有状態、各内部モジュール、非同期処理、認証、モード遷移、プロセス制御境界、エラー、ログおよびテストを定義する。

## 2. 技術構成

| 項目 | 設計 |
|---|---|
| 言語 | Rust stable |
| 非同期ランタイム | Tokio |
| HTTP / WebSocket | Axum |
| JSON | Serde / serde_json |
| ログ | tracing / tracing-subscriber |
| UUID | uuid |
| エラー | thiserror |
| 設定 | 環境変数とTOML |
| テスト | cargo test、tokio test |

## 3. クレート構造

```text
crates/suica-core/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── config.rs
    ├── error.rs
    ├── state.rs
    ├── api/
    │   ├── mod.rs
    │   ├── health.rs
    │   ├── pairing.rs
    │   └── websocket.rs
    ├── remote/
    │   ├── mod.rs
    │   ├── command.rs
    │   ├── message.rs
    │   └── validator.rs
    ├── mode/
    │   ├── mod.rs
    │   ├── manager.rs
    │   └── state.rs
    └── system/
        ├── mod.rs
        ├── chromium.rs
        └── process.rs
```

モジュール構成図は[draw.io](diagrams/suica-core.drawio)の「Modules」ページに定義する。

## 4. AppState

```rust
pub struct AppState {
    pub config: Arc<Config>,
    pub clients: ClientRegistry,
    pub mode_manager: Arc<ModeManager>,
    pub event_tx: broadcast::Sender<ServerMessage>,
    pub token_store: Arc<dyn TokenStore>,
}
```

| フィールド | 用途 | 同期方式 |
|---|---|---|
| `config` | 不変設定 | `Arc` |
| `clients` | 接続中クライアント管理 | `RwLock<HashMap>` |
| `mode_manager` | モード状態と切り替え | 内部`Mutex` |
| `event_tx` | 全クライアントへの状態配信 | Tokio broadcast |
| `token_store` | ペアリング済みトークンの検証 | Trait実装内で同期 |

## 5. mainモジュール

### 5.1 起動処理

1. ログ出力を初期化する。
2. 設定ファイルと環境変数を読み込む。
3. 必須設定を検証する。
4. 現在のChromiumプロセスから初期モードを判定する。
5. `AppState`を生成する。
6. Axum Routerを構築する。
7. 指定アドレスでListenする。
8. SIGTERMまたはSIGINTでGraceful Shutdownする。

### 5.2 Router

| Method | Path | 用途 | 認証 |
|---|---|---|---|
| GET | `/health/live` | プロセス生存確認 | 不要、ループバックまたはLAN |
| GET | `/health/ready` | 操作受付可否 | 不要、ループバックまたはLAN |
| POST | `/api/v1/pair` | ワンタイムコード交換 | コード |
| GET | `/ws` | WebSocket Upgrade | role別 |
| GET | `/*path` | React静的ファイル | ループバックのみ |

## 6. configモジュール

```rust
pub struct Config {
    pub bind_address: IpAddr,
    pub port: u16,
    pub static_dir: PathBuf,
    pub home_url: Url,
    pub chromium_binary: PathBuf,
    pub profile_dir: PathBuf,
    pub command_timeout: Duration,
    pub pairing_enabled: bool,
}
```

優先順位は環境変数、TOML、既定値の順とする。秘密情報はTOMLへ直接保存せず、権限600の専用ファイルまたはOSの秘密情報領域を使用する。

| 設定 | 既定値 |
|---|---|
| bind_address | `0.0.0.0` |
| port | `3030` |
| static_dir | `/opt/suica-tv/tv` |
| home_url | `http://127.0.0.1:3030/` |
| chromium_binary | `/usr/bin/chromium` |
| profile_dir | `/var/lib/suica-tv/chromium-profile` |
| command_timeout | 10秒 |

## 7. apiモジュール

### 7.1 websocket

責務はUpgrade要求の検証、接続登録、送受信タスクの生成、切断時の解放である。業務判断は`remote`または`mode`へ委譲する。

接続ごとに次の2タスクを生成する。

- Receive Task: Socketから受信し、メッセージ解析とCommand Dispatcherを呼ぶ。
- Send Task: 個別送信キューとbroadcastを購読し、Socketへ送る。

どちらかが終了したらもう一方をキャンセルし、ClientRegistryから接続を削除する。個別送信キューの上限は32件とし、遅いクライアントは切断する。

### 7.2 health

- Liveはイベントループが動作していれば200を返す。
- ReadyはModeManagerが初期化済みで静的ファイルが読める場合に200を返す。
- Ready失敗時は503を返し、秘密情報を含まない理由コードを付ける。

### 7.3 pairing

- ペアリング無効時は404を返す。
- ワンタイムコードを定数時間比較で検証する。
- 成功時に256bitのランダムトークンを生成する。
- 保存時はトークンそのものではなくSHA-256ハッシュを保存する。
- レスポンスで平文トークンを返すのは発行時の1回だけとする。

## 8. remoteモジュール

### 8.1 command

```rust
pub enum NavigationAction { Up, Down, Left, Right, Select, Back, Home }
pub enum SystemAction { SwitchMode { mode: DisplayMode } }
pub enum RemoteAction { Navigation(NavigationAction), System(SystemAction) }

pub struct RemoteCommand {
    pub request_id: Uuid,
    pub action: RemoteAction,
}
```

Serdeの独自デシリアライズまたはDTO変換により、文字列形式のAPIを型安全なenumへ変換する。未知の操作は`InvalidCommand`とする。

### 8.2 validator

| 検証 | 失敗時 |
|---|---|
| typeが`remote.command` | `invalid_message` |
| requestIdがUUID | `invalid_message` |
| actionが許可リスト内 | `invalid_command` |
| paramsが操作と一致 | `invalid_message` |
| Remoteロールからの送信 | `unauthorized` |
| 1メッセージ64KiB以内 | 接続終了1009 |

### 8.3 dispatcher

| 操作 | 条件 | 処理 |
|---|---|---|
| Navigation | TVクライアント接続中、TVモード | TVへ転送 |
| Home | 任意モード | TVモード保証後、ホームURLを表示 |
| SwitchMode | ModeManagerがidle | ModeManagerへ委譲 |

同じ`requestId`を過去60秒以内に処理済みの場合は、保存済み結果を返して二重実行を防止する。キャッシュ上限は1000件とする。

## 9. modeモジュール

### 9.1 状態

```rust
pub enum ModeState {
    Initializing,
    Stable(DisplayMode),
    Switching { from: DisplayMode, to: DisplayMode, request_id: Uuid },
    Degraded { last_stable: Option<DisplayMode>, reason: String },
}
```

### 9.2 ModeManager

```rust
impl ModeManager {
    pub async fn current(&self) -> ModeState;
    pub async fn switch(&self, target: DisplayMode, request_id: Uuid)
        -> Result<DisplayMode, CoreError>;
    pub async fn ensure_tv_home(&self, request_id: Uuid)
        -> Result<(), CoreError>;
    pub async fn reconcile(&self) -> Result<DisplayMode, CoreError>;
}
```

切り替え処理全体を`Mutex`で直列化する。現在と同じモードへの要求は冪等な成功とする。異なる切り替えの実行中は`busy`を返す。

### 9.3 TVモードへの遷移

1. 状態を`Switching`へ変更する。
2. 既存の管理対象Chromiumを確認する。
3. 存在しなければキオスク引数で起動する。
4. DevToolsを使わず、プロセス生存とHTTPヘルスを確認する。
5. ホームURLが表示されるよう起動引数または再起動で保証する。
6. カーソル非表示処理を有効化する。
7. 状態を`Stable(Tv)`へ更新する。
8. `system.state`をbroadcastする。

### 9.4 PCモードへの遷移

1. 状態を`Switching`へ変更する。
2. PIDファイルの管理対象ChromiumへSIGTERMを送る。
3. 最大5秒待機する。
4. 終了しない場合はSIGKILLを送り、さらに1秒確認する。
5. カーソル非表示処理を解除する。
6. デスクトップセッションの生存を確認する。
7. 状態を`Stable(Pc)`へ更新する。
8. `system.state`をbroadcastする。

切り替えシーケンスは[draw.io](diagrams/suica-core.drawio)の「Mode Sequence」ページに定義する。

## 10. systemモジュール

### 10.1 ProcessRunner Trait

```rust
#[async_trait]
pub trait ProcessRunner: Send + Sync {
    async fn spawn(&self, spec: ProcessSpec) -> Result<ManagedProcess, SystemError>;
    async fn terminate(&self, pid: u32, timeout: Duration) -> Result<(), SystemError>;
    async fn is_running(&self, pid: u32) -> Result<bool, SystemError>;
}
```

テストではFakeProcessRunnerへ置き換え、実際のChromiumを起動しない。

### 10.2 ChromiumController

```rust
pub struct ChromiumController {
    runner: Arc<dyn ProcessRunner>,
    binary: PathBuf,
    profile_dir: PathBuf,
    pid_file: PathBuf,
    home_url: Url,
}
```

- 任意のシェル文字列を組み立てず、`Command::arg`で固定引数を渡す。
- PIDファイルと`/proc/{pid}/cmdline`の両方を確認してから終了する。
- Suica TV専用プロファイルを使用し、通常のデスクトップChromiumを終了しない。
- 標準出力と標準エラーはjournaldへ送る。

## 11. ClientRegistry

| 項目 | 設計 |
|---|---|
| Remote接続数 | 最大4 |
| TV接続数 | 最大1。新接続時に古い接続を閉じる。 |
| 接続ID | UUID v4 |
| Keepalive | 20秒ごとにPing、10秒以内にPongがなければ切断 |
| 送信キュー | 接続ごとに32メッセージ |

## 12. エラー型

```rust
pub enum CoreError {
    InvalidMessage,
    InvalidCommand,
    Unauthorized,
    Busy,
    TvClientUnavailable,
    ModeSwitchFailed,
    ProcessControlFailed,
    Internal,
}
```

内部エラーの詳細はログだけに記録し、クライアントへは安全なコードと説明を返す。

## 13. ログとメトリクス

構造化ログへ次を含める。

| フィールド | 内容 |
|---|---|
| `event` | イベント名 |
| `request_id` | 要求ID |
| `connection_id` | 接続ID |
| `client_role` | remoteまたはtv |
| `mode_from` / `mode_to` | モード遷移 |
| `duration_ms` | 処理時間 |
| `result` | successまたはerror code |

トークン、ペアリングコード、完全なAuthorizationヘッダーは出力しない。

## 14. Graceful Shutdown

1. 新規HTTP接続を停止する。
2. 接続中クライアントへ`server.shutdown`を通知する。
3. 最大3秒、送信キューの完了を待つ。
4. WebSocketを1001 Going Awayで閉じる。
5. 実行中のモード切り替えを最大10秒待つ。
6. 状態をログへ記録して終了する。

ChromiumはTV表示継続のため、Core終了時には停止しない。

## 15. テスト設計

| ID | 対象 | 確認内容 |
|---|---|---|
| CO-U-01 | Validator | 未定義コマンドを拒否する。 |
| CO-U-02 | Validator | 不正UUIDと不正paramsを拒否する。 |
| CO-U-03 | Dispatcher | NavigationをTVだけへ転送する。 |
| CO-U-04 | Dispatcher | 重複requestIdで二重実行しない。 |
| CO-U-05 | ModeManager | 同一モード要求を冪等成功にする。 |
| CO-U-06 | ModeManager | 同時切り替えを直列化または拒否する。 |
| CO-U-07 | ChromiumController | 専用プロセスだけを終了する。 |
| CO-U-08 | Pairing | 期限切れコードを拒否する。 |
| CO-I-01 | WebSocket | RemoteからTVへ命令を中継する。 |
| CO-I-02 | WebSocket | 無効トークンを401で拒否する。 |
| CO-I-03 | Mode | FakeProcessで成功後に状態を通知する。 |
| CO-I-04 | Mode | プロセス失敗時に旧状態を維持する。 |

## 16. 受入基準

- 許可された命令だけを処理できる。
- RemoteとTVの切断がCore全体を停止させない。
- モード状態はOS操作の成功後にのみ更新される。
- 同時操作でChromiumプロセスが重複起動しない。
- Core再起動後に実プロセスから状態を復元できる。
