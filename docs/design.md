# Suica TV v0.1 基本設計書

| 項目 | 内容 |
|---|---|
| 文書名 | Suica TV v0.1 基本設計書 |
| バージョン | 0.1 |
| 作成日 | 2026年9月21日 |
| 対象システム | Suica TV v0.1 |
| 関連図 | [`diagrams/suica-tv-v0.1.drawio`](diagrams/suica-tv-v0.1.drawio) |

## 1. プロジェクト概要

### 1.1 目的

Raspberry Pi 5を家庭用テレビに接続し、iPhoneから操作できるテレビ用PC「Suica TV」を開発する。

通常時は動画視聴に適したテレビ向けホーム画面を表示し、必要に応じてRaspberry Pi OSの通常デスクトップへ切り替えられるようにする。

### 1.2 開発方針

- テレビ用画面はReactとTypeScriptで開発する。
- iPhone用リモコンはSwiftとSwiftUIで開発する。
- Raspberry Pi側の操作管理サービスはRustで開発する。
- OSはRaspberry Pi OS 64-bit Desktopを使用する。
- Webサイトは既存のChromiumで表示する。
- v0.1ではブラウザエンジンおよびOSを自作しない。
- 将来の機能追加に備え、画面表示と操作管理を分離する。

### 1.3 対象ハードウェアとソフトウェア

| 項目 | 構成 |
|---|---|
| 本体 | Raspberry Pi 5（8GB） |
| OS | Raspberry Pi OS 64-bit Desktop |
| ディスプレイ | HDMI接続のテレビ |
| リモコン | iPhone |
| 通信 | 家庭内Wi-Fi |
| Webブラウザ | Chromium |

### 1.4 前提条件

- Raspberry PiとiPhoneは同じ家庭内ネットワークへ接続する。
- Raspberry Pi OSはデスクトップセッションへ自動ログインできる。
- PCモードの操作には物理キーボード・マウス、またはKDE Connectを使用できる。
- 外部Webサイト内の独自リモコン操作はv0.1の保証対象外とする。
- Suica CoreのAPIは家庭内ネットワーク外へ公開しない。

## 2. v0.1 開発スコープ

v0.1では、テレビ用ホーム画面、iPhone用リモコン、TVモードとPCモードの切り替えを実装する。

### 2.1 F-001 React製ホーム画面

Raspberry Piの起動後、テレビにSuica TVのホーム画面を表示する。

| ID | 機能要件 |
|---|---|
| F-001-01 | ReactとTypeScriptでホーム画面を構築する。 |
| F-001-02 | Chromiumでホーム画面を全画面表示する。 |
| F-001-03 | YouTubeなどのWebサイトを開ける。 |
| F-001-04 | 上下左右キーでメニューを選択できる。 |
| F-001-05 | Enterキーで選択中のメニューを実行できる。 |
| F-001-06 | Escapeキーで前の画面へ戻れる。 |
| F-001-07 | 選択中のメニューを視覚的に強調する。 |
| F-001-08 | テレビに接続したキーボードでも同じ操作ができる。 |

ホーム画面には次のメニューを配置する。

| メニュー | 動作 |
|---|---|
| YouTube | YouTubeのテレビ向けWeb画面を開く。 |
| Browser | Webブラウザ画面を開く。 |
| PC Mode | PCモードへの切り替えを要求する。 |
| Settings | Suica TVの設定画面を開く。 |

画面構成はdraw.ioファイルの「TV Home」ページに定義する。

### 2.2 F-002 iPhone専用リモコン

SwiftUIでiPhone用リモコンアプリ「Suica Remote」を開発する。

| ID | 機能要件 |
|---|---|
| F-002-01 | 家庭内Wi-Fi経由でRaspberry Piへ接続する。 |
| F-002-02 | 上下左右ボタンでメニューを操作する。 |
| F-002-03 | 決定ボタンで選択中のメニューを実行する。 |
| F-002-04 | 戻るボタンで前の画面へ戻る。 |
| F-002-05 | ホームボタンでSuica TVホームへ戻る。 |
| F-002-06 | Raspberry Piとの接続状態を表示する。 |
| F-002-07 | TVモードとPCモードの切り替え操作を提供する。 |
| F-002-08 | 切断後に自動再接続する。 |

リモコン画面はdraw.ioファイルの「iPhone Remote」ページに定義する。

### 2.3 F-003 表示モード切り替え

同じRaspberry Pi OS上で、Chromiumのキオスク表示と通常のデスクトップ表示を切り替える。OSの再起動やmicroSDカードの交換は行わない。

#### TVモード

- React製ホーム画面をChromiumのキオスクモードで前面表示する。
- 十字キーと決定ボタンを中心に操作する。
- 通常のLinuxデスクトップは背後に維持する。
- マウスポインターは通常非表示とする。

#### PCモード

- キオスク用Chromiumを終了または非表示にする。
- Raspberry Pi OSの通常デスクトップを表示する。
- Chromiumなどのアプリケーションをウィンドウ表示で利用できる。
- 物理キーボードとマウスを利用できる。
- Suica RemoteからTVモードへ戻れる。

## 3. システムアーキテクチャ

システム構成図はdraw.ioファイルの「Architecture」ページに定義する。

### 3.1 コンポーネント責務

| コンポーネント | 技術 | 責務 |
|---|---|---|
| Suica TV Home | React / TypeScript | 画面表示、フォーカス管理、メニュー実行、画面遷移 |
| Suica Remote | Swift / SwiftUI | リモコンUI、操作命令送信、接続状態とモード状態の表示 |
| Suica Core | Rust / Tokio / Axum | 接続認証、命令検証、命令中継、モード管理、Chromium制御 |
| Chromium | 既存ソフトウェア | React画面および外部Webサイトの表示 |
| Raspberry Pi OS | Linux | デスクトップ、プロセス、ネットワーク、デバイス管理 |

### 3.2 責務分離

- 画面固有のフォーカス移動と画面遷移はSuica TV Homeが担当する。
- OSやプロセスに影響する操作はSuica Coreが担当する。
- Suica Remoteは操作要求を送信し、Suica Coreから受信した確定状態を表示する。
- Suica Remoteだけが先行してモード表示を変更してはならない。

## 4. コンポーネント間通信

### 4.1 通信方式

| 通信経路 | 方式 | 用途 |
|---|---|---|
| Suica Remote → Suica Core | WebSocket | 操作命令の送信、処理結果と状態の受信 |
| Suica Core → Suica TV Home | WebSocket | ナビゲーション命令と状態の配信 |
| Suica TV Home → 外部サイト | HTTPS | 動画サービス、一般Webサイトの表示 |

初期バージョンは家庭内LANでの利用に限定する。WebSocket接続が切断された場合、iPhone側は一定間隔で自動再接続する。再接続後、Suica Coreは現在のモードを再通知する。

### 4.2 共通メッセージ形式

- 文字コードはUTF-8とする。
- メッセージはJSONテキストとする。
- クライアントが送信する要求には`requestId`を付与する。
- `requestId`はUUID形式とし、要求と応答の対応付けに使用する。

```json
{
  "type": "remote.command",
  "requestId": "550e8400-e29b-41d4-a716-446655440000",
  "action": "navigation.up"
}
```

### 4.3 操作命令

| 操作名 | 説明 | 処理主体 |
|---|---|---|
| `navigation.up` | 上へ移動 | Suica TV Home |
| `navigation.down` | 下へ移動 | Suica TV Home |
| `navigation.left` | 左へ移動 | Suica TV Home |
| `navigation.right` | 右へ移動 | Suica TV Home |
| `navigation.select` | 決定 | Suica TV Home |
| `navigation.back` | 前の画面へ戻る | Suica TV Home |
| `navigation.home` | ホーム画面へ戻る | Suica TV Home / Suica Core |
| `system.switch_mode` | 表示モードを切り替える | Suica Core |

モード切り替え要求は次の形式とする。

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

`params.mode`には`tv`または`pc`を指定する。

### 4.4 状態通知

```json
{
  "type": "system.state",
  "mode": "tv"
}
```

モード変更時は、Suica RemoteとSuica TV Homeの両方へ確定した状態を通知する。Suica CoreはChromiumの起動・終了結果を確認し、成功した場合にのみ状態を更新する。

### 4.5 処理結果

成功時:

```json
{
  "type": "command.result",
  "requestId": "550e8400-e29b-41d4-a716-446655440000",
  "ok": true
}
```

失敗時:

```json
{
  "type": "command.result",
  "requestId": "550e8400-e29b-41d4-a716-446655440000",
  "ok": false,
  "error": {
    "code": "mode_switch_failed",
    "message": "Chromiumの終了を確認できませんでした。"
  }
}
```

### 4.6 エラーコード

| コード | 意味 |
|---|---|
| `invalid_message` | JSON形式または必須項目が不正 |
| `invalid_command` | 未定義の操作 |
| `unauthorized` | トークンが無効 |
| `invalid_state` | 現在状態では実行できない操作 |
| `mode_switch_failed` | 表示モード切り替え処理に失敗 |
| `tv_client_unavailable` | Suica TV Homeへ命令を転送できない |
| `internal_error` | Suica Core内部エラー |

### 4.7 接続と再接続

1. Suica RemoteがSuica CoreへWebSocket接続を開始する。
2. Suica Coreがペアリングトークンを検証する。
3. 検証成功後、Suica Coreが現在の`system.state`を送信する。
4. Suica Remoteは接続中表示へ切り替え、操作を有効化する。
5. 切断時、Suica Remoteは操作を無効化して再接続を開始する。
6. 再接続間隔は指数バックオフとし、1秒、2秒、4秒、8秒、最大10秒とする。

## 5. 画面設計

### 5.1 TVモード ホーム画面

| 項目 | 設計 |
|---|---|
| 初期フォーカス | YouTube |
| フォーカス移動 | 2列グリッド上を上下左右に移動し、端ではその場に留まる。 |
| 選択表示 | 枠線、背景色、拡大のうち2種類以上で明確に表示する。 |
| 決定 | フォーカス中の項目に対応する処理を実行する。 |
| 戻る | 子画面では直前画面へ戻る。ホーム画面では何もしない。 |
| ホーム | ホーム画面へ戻り、初期フォーカスへ復帰する。 |

### 5.2 iPhone リモコン画面

画面には次の要素を表示する。

- アプリ名
- Raspberry Piとの接続状態
- 上下左右ボタン
- 決定ボタン
- 戻るボタン
- ホームボタン
- 設定画面への導線

未接続時は接続状態を明示し、操作ボタンを無効化する。

### 5.3 設定画面

| 項目 | 内容 |
|---|---|
| 接続先 | Raspberry Piのホスト名またはIPアドレス |
| 接続状態 | 未接続、接続中、接続済み、再接続中 |
| 現在のモード | Suica Coreから通知された`tv`または`pc` |
| TVモードボタン | TVモードへの切り替えを要求する。 |
| PCモードボタン | PCモードへの切り替えを要求する。 |
| 戻るボタン | リモコン画面へ戻る。 |

### 5.4 PCモード

Raspberry Pi OSの通常デスクトップを表示する。PCモードの操作には物理キーボード・マウス、またはKDE Connectを使用する。独自タッチパッドと仮想キーボードはv0.1の対象外とする。

## 6. Suica Core 設計

### 6.1 使用技術

| 項目 | 採用技術 |
|---|---|
| 言語 | Rust |
| 非同期ランタイム | Tokio |
| HTTP / WebSocket | Axum |
| データ形式 | JSON |
| 動作環境 | Raspberry Pi OS 64-bit |

### 6.2 内部モジュール

```text
suica-core/
└── src/
    ├── main.rs
    ├── api/
    │   ├── mod.rs
    │   └── websocket.rs
    ├── remote/
    │   ├── mod.rs
    │   └── command.rs
    ├── mode/
    │   ├── mod.rs
    │   └── manager.rs
    └── system/
        ├── mod.rs
        └── chromium.rs
```

| モジュール | 責務 |
|---|---|
| `api` | WebSocket接続の受付、認証、メッセージ送受信 |
| `remote` | JSON解析、コマンド許可リスト検証 |
| `mode` | 現在モードの保持、排他的な切り替え、状態通知 |
| `system` | Chromiumプロセスの起動、終了、結果確認 |
| `main` | 設定読込、共有状態生成、サーバー起動 |

### 6.3 基本処理

1. Suica Remoteから操作命令を受信する。
2. メッセージ形式、トークン、操作の許可リストを検証する。
3. 現在の表示モードと操作の実行可否を確認する。
4. ナビゲーション操作はSuica TV Homeへ転送する。
5. モード切り替えはChromiumの表示状態を変更する。
6. 実処理の成功を確認して状態を更新する。
7. 要求元へ処理結果を返し、接続中のクライアントへ状態を配信する。

### 6.4 モード状態

```text
起動中 ──起動成功──▶ TV
  │                   │
  └──起動失敗──▶ エラー

TV ──PC切替成功──▶ PC
▲                  │
└──TV切替成功──────┘
```

状態遷移中の追加切り替え要求は直列化する。切り替えに失敗した場合は直前の確定状態を維持する。

### 6.5 異常処理

| 事象 | 処理 |
|---|---|
| 同時モード切り替え | 処理を直列化し、実行中の追加要求は待機または拒否する。 |
| 不正JSON | 接続を維持したままエラーを返す。連続する場合は切断する。 |
| 未定義コマンド | 実行せず`invalid_command`を返す。 |
| Chromium起動失敗 | モードを更新せず、失敗結果を返してログへ記録する。 |
| Suica Remote切断 | TV表示を維持し、再接続を待つ。 |
| Suica TV Home切断 | 命令転送を停止し、再接続後に現在状態を通知する。 |

## 7. 起動・終了設計

起動フローおよびモード切り替えシーケンスはdraw.ioファイルの「Startup Flow」と「Mode Switch」ページに定義する。

### 7.1 Raspberry Pi起動時

1. Raspberry Pi OSを起動する。
2. デスクトップセッションを開始する。
3. systemdがSuica Coreを起動する。
4. Suica CoreがReactホーム画面の配信を開始する。
5. Chromiumをキオスクモードで起動する。
6. TVモードの表示完了を確認する。
7. Suica Remoteからの接続を受け付ける。

### 7.2 PCモードへの切り替え

1. iPhoneでPCモードを選択する。
2. Suica Coreが命令とトークンを検証する。
3. キオスク用Chromiumを終了または非表示にする。
4. デスクトップが操作可能であることを確認する。
5. 現在モードを`pc`へ更新する。
6. 接続中のクライアントへ状態を通知する。

### 7.3 TVモードへの復帰

1. iPhoneでTVモードを選択する。
2. Suica Coreが命令とトークンを検証する。
3. Chromiumをキオスクモードで起動または前面表示する。
4. Reactホーム画面の表示を確認する。
5. 現在モードを`tv`へ更新する。
6. 接続中のクライアントへ状態を通知する。

### 7.4 自動復旧

- Suica Coreが異常終了した場合、systemdが再起動する。
- Raspberry Pi再起動後は同じ起動順序でTVモードへ復旧する。
- Chromiumだけが終了した場合、Suica Coreが状態不一致を検知して再起動する。
- 再起動を繰り返す場合は一定回数で停止し、ログへ原因を記録する。

## 8. ディレクトリ構成

```text
suica-tv/
├── apps/
│   ├── tv/
│   │   ├── src/
│   │   │   ├── components/
│   │   │   ├── pages/
│   │   │   ├── hooks/
│   │   │   └── services/
│   │   ├── package.json
│   │   └── vite.config.ts
│   └── ios/
│       ├── SuicaRemote/
│       │   ├── Views/
│       │   ├── Models/
│       │   ├── Services/
│       │   └── SuicaRemoteApp.swift
│       └── SuicaRemote.xcodeproj
├── crates/
│   └── suica-core/
│       ├── src/
│       │   ├── api/
│       │   ├── remote/
│       │   ├── mode/
│       │   ├── system/
│       │   └── main.rs
│       └── Cargo.toml
├── contracts/
│   └── remote-api.md
├── scripts/
│   ├── setup.sh
│   ├── start-tv.sh
│   └── start-pc.sh
├── docs/
│   ├── design.md
│   └── diagrams/
│       └── suica-tv-v0.1.drawio
└── README.md
```

通信メッセージの定義は`contracts`に集約し、React、Swift、Rustで同じ操作名と状態値を使用する。OS依存処理は`scripts`とRustの`system`モジュールへ閉じ込める。

## 9. 開発計画

### 9.1 Phase 1 Reactホーム画面

- ReactとTypeScriptのプロジェクトを作成する。
- テレビ用ホーム画面を作成する。
- キーボードの上下左右とEnterで操作できるようにする。
- Chromiumで全画面表示できることを確認する。

**完了条件:** キーボードだけでホーム画面を操作できる。

### 9.2 Phase 2 Rust製Suica Core

- AxumでWebSocketサーバーを作成する。
- リモコン操作命令を受信する。
- Suica TV Homeへ操作命令を転送する。
- Chromiumの起動・終了処理を実装する。
- 表示モードの状態管理を実装する。

**完了条件:** Suica Core経由でReact画面を操作し、TVモードとPCモードを切り替えられる。

### 9.3 Phase 3 Swift製リモコン

- SwiftUIで十字キー型リモコンを作成する。
- Raspberry PiとのWebSocket通信を実装する。
- 上下左右、決定、戻る、ホームを実装する。
- 設定画面からモードを切り替えられるようにする。
- Raspberry Piとの接続状態を表示する。

**完了条件:** iPhoneからReactホーム画面を操作し、TVモードとPCモードを切り替えられる。

### 9.4 Phase 4 統合試験

- Raspberry Pi起動時のホーム画面自動表示
- iPhoneからのメニュー操作
- WebSocket切断後の再接続
- TVモードからPCモード、PCモードからTVモードへの切り替え
- 外部サイトを開いた後のホーム復帰
- Raspberry Pi再起動後の自動復旧

**完了条件:** 通常利用時にキーボードを接続しなくても、iPhoneからホーム画面を操作・復帰できる。

## 10. v0.1 対象外

| 機能 | 将来の実装方針 |
|---|---|
| HDMI入力切り替え | HDMI-CECまたは赤外線制御 |
| Switchとの連携 | HDMI入力制御モジュール |
| Zaim家計管理 | Web版起動またはAPI連携 |
| 家族会議の録音 | Swiftのマイク録音 |
| AI文字起こし・議事録 | 音声認識とLLM |
| 写真・動画共有 | メディア管理モジュール |
| 独自タッチパッド | SwiftとLinux入力制御 |
| ゲームコントローラー | Rustとevdev |
| 独自ブラウザ | 既存ブラウザエンジンの組み込み |

将来機能はSuica Coreの独立モジュールとして追加し、既存のリモコン操作とホーム画面への影響を最小化する。

## 11. 非機能設計

### 11.1 セキュリティ

- 受け付ける操作は定義済みコマンドに限定する。
- iPhoneから任意のシェルコマンドを実行できないようにする。
- 初回接続時にペアリングトークンを発行する。
- トークンを検証できない端末の接続と操作を拒否する。
- APIを家庭内ネットワーク外へ公開しない。
- 将来のHTTPSおよびWSS導入を妨げない構成とする。
- ログへペアリングトークンや個人情報を出力しない。

### 11.2 可用性と復旧性

- Suica Coreは異常終了時に自動再起動する。
- Suica Remoteは切断後に自動再接続する。
- 再接続後はSuica Coreの確定状態を取得し直す。
- Raspberry Pi再起動後はTVモードへ自動復旧する。

### 11.3 性能目標

| 項目 | 目標 |
|---|---|
| リモコン操作応答 | 家庭内LANの通常状態で、操作から画面反映まで500ミリ秒以内を目安とする。 |
| モード切り替え | 要求から表示完了まで10秒以内を目標とする。 |
| 再接続 | Wi-Fi復旧後、ユーザー操作なしで10秒以内に再接続を試行する。 |

### 11.4 ログ

Suica Coreは、起動、接続、認証失敗、コマンド種別、モード切り替え開始・結果、Chromium制御失敗を記録する。通常ログに秘密情報を出力しない。

## 12. 試験方針

| 試験区分 | 主な確認内容 |
|---|---|
| 単体試験 | コマンド解析、許可リスト、フォーカス移動、モード状態遷移 |
| 結合試験 | SwiftからCoreへの送信、CoreからReactへの転送、状態通知 |
| 実機試験 | Raspberry Pi起動、Chromium全画面、HDMI表示、iPhone接続 |
| 障害試験 | Wi-Fi切断、Core停止、Chromium停止、再起動後の復旧 |
| セキュリティ試験 | 無効トークン、未定義コマンド、不正JSON、LAN外からの到達性 |

## 13. v0.1 完成条件

- [ ] Raspberry Piを起動するとReactホーム画面が自動表示される。
- [ ] iPhoneのSwift製リモコンからRaspberry Piへ接続できる。
- [ ] 十字キーでホーム画面のメニューを移動できる。
- [ ] 決定ボタンで選択中のメニューを実行できる。
- [ ] ホームボタンでReactホーム画面へ戻れる。
- [ ] TVモードからPCモードへ切り替えられる。
- [ ] PCモードからTVモードへ戻れる。
- [ ] 通信切断後にiPhoneリモコンが再接続できる。
- [ ] Raspberry Piを再起動してもSuica TVが自動復旧する。

## 14. 最初の開発目標

iPhoneのSwift製リモコンからRust製Suica Coreを経由してReactのテレビ画面を操作し、TVモードとPCモードを切り替えられるSuica TV v0.1を完成させる。
