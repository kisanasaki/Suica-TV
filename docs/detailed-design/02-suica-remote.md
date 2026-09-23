# Suica Remote 詳細設計書

## 1. 目的と範囲

Suica Remoteは、iPhoneからSuica TVを操作するSwiftUIアプリケーションである。本書は画面、状態管理、WebSocket接続、再接続、コマンド送信、設定保存、ペアリング、エラー処理およびテストを定義する。

PCモード向けのタッチパッド、文字入力、音声入力はv0.1の対象外とする。

## 2. 技術構成

| 項目 | 設計 |
|---|---|
| 言語 | Swift |
| UI | SwiftUI |
| 最低OS | iOS 17 |
| 通信 | `URLSessionWebSocketTask` |
| 非同期処理 | Swift Concurrency、`async` / `await` |
| 状態管理 | `@Observable`の`RemoteViewModel` |
| 設定保存 | UserDefaults |
| トークン保存 | Keychain |
| ログ | `OSLog` |
| テスト | XCTest |

## 3. ディレクトリ構成

```text
apps/ios/SuicaRemote/
├── App/
│   └── SuicaRemoteApp.swift
├── Views/
│   ├── RemoteView.swift
│   ├── DirectionPad.swift
│   ├── ConnectionStatusView.swift
│   └── SettingsView.swift
├── ViewModels/
│   ├── RemoteViewModel.swift
│   └── SettingsViewModel.swift
├── Models/
│   ├── RemoteCommand.swift
│   ├── ServerMessage.swift
│   ├── DisplayMode.swift
│   └── ConnectionState.swift
├── Services/
│   ├── WebSocketClient.swift
│   ├── ReconnectPolicy.swift
│   ├── PairingService.swift
│   └── KeychainStore.swift
├── Configuration/
│   └── AppSettings.swift
└── Resources/
    ├── Assets.xcassets
    └── Localizable.xcstrings
```

## 4. 主要型

```swift
enum DisplayMode: String, Codable {
    case tv
    case pc
}

enum ConnectionState: Equatable {
    case disconnected
    case connecting
    case connected
    case reconnecting(attempt: Int)
    case failed(message: String)
}

enum RemoteAction: String, Codable {
    case up = "navigation.up"
    case down = "navigation.down"
    case left = "navigation.left"
    case right = "navigation.right"
    case select = "navigation.select"
    case back = "navigation.back"
    case home = "navigation.home"
    case switchMode = "system.switch_mode"
    case scroll = "pointer.scroll"
    case inputText = "input.text"
    case deleteBackward = "input.delete_backward"
    case submitText = "input.submit"
}

struct RemoteCommand: Encodable {
    let type = "remote.command"
    let requestId: UUID
    let action: RemoteAction
    let params: CommandParams?
}
```

## 5. 画面設計

### 5.1 RemoteView

| UI要素 | 動作 |
|---|---|
| 接続状態 | 現在の`ConnectionState`を文字と色で表示する。 |
| 上下左右 | 対応する`navigation.*`を1回送信する。 |
| OK | `navigation.select`を送信する。 |
| スクロールパッド | ドラッグ量を75ミリ秒単位でまとめ、`pointer.scroll`を送信する。 |
| 戻る | `navigation.back`を送信する。 |
| ホーム | `navigation.home`を送信する。PCモード時も有効とする。 |
| 設定 | SettingsViewをモーダル表示する。 |
| 文字入力 | TextInputViewをモーダル表示する。 |

未接続中は設定ボタンを除く操作を無効化する。ボタン押下時は軽い触覚フィードバックを発生させる。

### 5.2 TextInputView

日本語IMEの変換途中では送信せず、「文字を送信」を押した時点の確定文字列だけを送信する。
入力は1〜200文字かつ改行なしとする。誤入力を防ぐため、利用者がテレビ画面の検索欄を
選択済みであることを確認するまで、送信・削除・検索実行を無効化する。

| UI要素 | 動作 |
|---|---|
| 文字を送信 | `input.text`を送信し、成功した送信開始後に下書きを消去する。 |
| 1文字削除 | `input.delete_backward`を送信する。 |
| 検索を実行 | `input.submit`を送信する。 |
| キャンセル | 送信せず画面を閉じる。 |

### 5.3 SettingsView

| UI要素 | 入力規則 |
|---|---|
| Raspberry Piホスト | ホスト名またはIPv4アドレス。スキームやパスは入力させない。 |
| ポート | 初期値3030。1から65535。 |
| ペアリング | トークン未登録時に表示する。 |
| 現在モード | Coreから受信した確定状態を表示する。 |
| TVモード | 接続中かつ現在がPCの場合に有効。 |
| PCモード | 接続中かつ現在がTVの場合に有効。 |
| 再接続 | 設定を保存してWebSocketを再生成する。 |

画面構成は[draw.io](diagrams/suica-remote.drawio)の「Screens」ページに定義する。

## 6. ViewModel設計

### 6.1 RemoteViewModel

| プロパティ | 型 | 初期値 | 説明 |
|---|---|---|---|
| `connectionState` | `ConnectionState` | `.disconnected` | 接続状態 |
| `displayMode` | `DisplayMode?` | `nil` | Coreが通知した確定モード |
| `pendingRequests` | `[UUID: PendingRequest]` | 空 | 応答待ち要求 |
| `alertMessage` | `String?` | `nil` | ユーザー向けエラー |
| `isSettingsPresented` | `Bool` | `false` | 設定画面表示状態 |

| メソッド | 処理 |
|---|---|
| `start()` | 設定を読み、接続を開始する。 |
| `stop()` | 自動再接続を停止し、Socketを閉じる。 |
| `sendNavigation(_:)` | 接続確認後に操作命令を送信する。 |
| `queueScroll(dx:dy:)` | 連続するドラッグ量を集約し、各軸±1200以内で送信する。 |
| `switchMode(to:)` | 二重送信を防ぎ、切り替え要求を送信する。 |
| `handle(_:)` | ServerMessageを状態へ反映する。 |
| `retry()` | 現在の設定で即時再接続する。 |

UI更新を行うViewModelは`@MainActor`とする。WebSocketClientはActorとして実装し、送受信を直列化する。

## 7. WebSocketClient設計

```swift
actor WebSocketClient {
    func connect(configuration: ConnectionConfiguration) async throws
    func disconnect() async
    func send(_ command: RemoteCommand) async throws
    func messages() -> AsyncThrowingStream<ServerMessage, Error>
}
```

### 7.1 接続URL

```text
ws://{host}:{port}/ws?role=remote&protocolVersion=1
```

トークンはURLへ含めず、HTTP Upgrade要求の`Authorization: Bearer {token}`ヘッダーで送信する。

### 7.2 接続状態遷移

| 現在状態 | イベント | 次状態 |
|---|---|---|
| disconnected | `start` | connecting |
| connecting | 接続成功 | connected |
| connecting | 接続失敗 | reconnecting |
| connected | Socket切断 | reconnecting |
| reconnecting | タイマー満了 | connecting |
| 任意 | `stop` | disconnected |
| 任意 | 認証失敗 | failed |

認証失敗は設定変更が必要なため、自動再接続しない。状態遷移図は[draw.io](diagrams/suica-remote.drawio)の「Connection State」ページに定義する。

### 7.3 再接続ポリシー

| 試行回数 | 待機時間 |
|---:|---:|
| 1 | 1秒 |
| 2 | 2秒 |
| 3 | 4秒 |
| 4 | 8秒 |
| 5以降 | 10秒 |

- 待機時間へ0から500ミリ秒のジッターを加える。
- 接続が30秒継続した時点で試行回数を0へ戻す。
- アプリがバックグラウンドへ移行した場合は既存接続を維持するが、OSによる停止を許容する。
- フォアグラウンド復帰時にSocket状態を確認し、未接続なら即時接続する。

## 8. コマンド送信

1. 接続状態が`connected`であることを確認する。
2. UUIDを生成して`requestId`へ設定する。
3. 送信前に`pendingRequests`へ登録する。
4. JSONへエンコードして送信する。
5. `command.result`受信時に同じIDを削除する。
6. ナビゲーションは5秒、モード切り替えは12秒以内に応答がなければタイムアウトとし、ユーザーへ通知する。

ナビゲーションコマンドは応答待ちでも次を送信できる。モード切り替えは、前の切り替え要求が完了するまで再送信できない。

## 9. 受信メッセージ処理

| type | 処理 |
|---|---|
| `system.state` | `displayMode`を更新する。 |
| `command.result` | 対応する要求を完了し、失敗ならアラートを設定する。 |
| `error` | エラーコードをユーザー向け文言へ変換する。 |
| `server.hello` | プロトコルバージョンを確認する。 |
| 未知 | 無視してデバッグログへ記録する。 |

## 10. 設定と秘密情報

| データ | 保存先 | 備考 |
|---|---|---|
| ホスト名 | UserDefaults | 初期値`raspberrypi.local` |
| ポート | UserDefaults | 初期値3030 |
| ペアリングトークン | Keychain | アプリ削除まで保持 |
| 最終モード | 保存しない | 接続時にCoreから取得する。 |

トークンは画面へ平文で再表示せず、登録済みかどうかだけ表示する。

## 11. ペアリング

v0.1のペアリングは、Raspberry Pi上で発行されたワンタイムコードをiPhoneへ入力する方式とする。

1. ユーザーがRaspberry Pi側でペアリングモードを有効化する。
2. 6桁コードをiPhoneへ入力する。
3. `POST /api/v1/pair`へコードと端末名を送信する。
4. Coreが長期トークンを返す。
5. アプリがトークンをKeychainへ保存する。
6. 以降のWebSocket接続でBearerトークンを使用する。

ワンタイムコードは10分で失効し、5回失敗した接続元を10分間拒否する。

## 12. エラー表示

| エラー | 表示 | 再試行 |
|---|---|---|
| ネットワーク未接続 | 「Wi-Fi接続を確認してください」 | 自動 |
| ホストへ到達不可 | 「Suica TVが見つかりません」 | 自動 |
| 認証失敗 | 「再ペアリングが必要です」 | 手動 |
| コマンドタイムアウト | 「操作を完了できませんでした」 | 次操作可 |
| モード切り替え失敗 | Coreの安全なメッセージを表示 | 手動 |
| プロトコル不一致 | 「アプリの更新が必要です」 | なし |

## 13. アクセシビリティ

- すべてのボタンへVoiceOverラベルとヒントを設定する。
- Dynamic Typeを許可し、最大サイズでもボタンが重ならないようにする。
- 色だけで接続状態を表現しない。
- 操作ボタンのタップ領域は44×44pt以上とする。
- 方向ボタンの配置は画面回転後も意味が変わらないよう固定する。

## 14. テスト設計

| ID | 対象 | 確認内容 |
|---|---|---|
| RM-U-01 | URL生成 | ホストとポートから正しいURLを生成する。 |
| RM-U-02 | エンコード | 全コマンドが契約どおりのJSONになる。 |
| RM-U-03 | デコード | 状態、成功、失敗を解析できる。 |
| RM-U-04 | 再接続 | 待機時間が1、2、4、8、10秒となる。 |
| RM-U-05 | ViewModel | 未接続時に操作を送信しない。 |
| RM-U-06 | ViewModel | モードは`system.state`受信後にのみ変わる。 |
| RM-U-07 | Keychain | トークンを平文設定へ保存しない。 |
| RM-I-01 | 結合 | Mock Coreへ接続して操作を送信できる。 |
| RM-I-02 | 結合 | Socket切断後に再接続する。 |
| RM-I-03 | 結合 | 認証失敗時に自動再接続を停止する。 |

## 15. 受入基準

- iPhoneから全ナビゲーション操作を送信できる。
- 接続状態と現在モードがCoreの状態と一致する。
- 一時的なWi-Fi切断後にユーザー操作なしで復旧する。
- PCモードでもTVモードへの復帰ボタンを利用できる。
- トークンがKeychain以外へ保存されない。
