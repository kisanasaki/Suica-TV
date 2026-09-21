# Suica TV Home 詳細設計書

## 1. 目的と範囲

Suica TV Homeは、テレビ画面へ表示するReactアプリケーションである。本書は画面構造、コンポーネント、状態、フォーカス移動、キー入力、WebSocket接続、外部サイト起動、例外処理およびテスト項目を定義する。

OSプロセスの起動・終了、認証、モード確定はSuica Coreの責務とし、本モジュールでは実行しない。

## 2. 技術構成

| 項目 | 設計 |
|---|---|
| 言語 | TypeScript |
| UI | React |
| ビルド | Vite |
| 状態管理 | React Contextと`useReducer`。外部ライブラリは使用しない。 |
| 通信 | Browser WebSocket API |
| スタイル | CSS Modulesまたは単一テーマCSS |
| テスト | Vitest、React Testing Library |
| 対象ブラウザ | Raspberry Pi OSにインストールされたChromium最新版 |

## 3. ディレクトリ構成

```text
apps/tv/
├── index.html
├── package.json
├── vite.config.ts
├── tsconfig.json
└── src/
    ├── main.tsx
    ├── App.tsx
    ├── components/
    │   ├── AppTile.tsx
    │   ├── ConnectionBadge.tsx
    │   └── FocusFrame.tsx
    ├── pages/
    │   ├── HomePage.tsx
    │   └── SettingsPage.tsx
    ├── hooks/
    │   ├── useKeyboardNavigation.ts
    │   └── useRemoteCommands.ts
    ├── services/
    │   ├── coreClient.ts
    │   └── launcher.ts
    ├── state/
    │   ├── appReducer.ts
    │   └── types.ts
    ├── config/
    │   └── menu.ts
    └── styles/
        ├── theme.css
        └── global.css
```

## 4. コンポーネント設計

| コンポーネント | Props | 責務 |
|---|---|---|
| `App` | なし | Context生成、ルーティング、Core接続開始 |
| `HomePage` | `items`, `selectedId` | メニューグリッド表示、選択項目の実行 |
| `SettingsPage` | `connection`, `mode` | バージョン、接続状態、現在モードの表示 |
| `AppTile` | `item`, `selected`, `onSelect` | メニュー1件の表示と強調 |
| `ConnectionBadge` | `status` | Core接続状態の表示 |
| `FocusFrame` | `active` | テレビ視聴距離でも判別できるフォーカス表現 |
| `useKeyboardNavigation` | `dispatch` | キー入力を内部アクションへ変換 |
| `useRemoteCommands` | `dispatch` | WebSocket操作命令を内部アクションへ変換 |
| `coreClient` | 接続設定 | WebSocket接続、再接続、メッセージ解析 |
| `launcher` | メニュー定義 | URL遷移またはCoreへのシステム操作要求 |

構成図は[draw.io](diagrams/tv-home.drawio)の「Components」ページに定義する。

## 5. データ型

```ts
type PageId = 'home' | 'settings';
type DisplayMode = 'tv' | 'pc';
type ConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'reconnecting';
type NavigationAction =
  | 'navigation.up'
  | 'navigation.down'
  | 'navigation.left'
  | 'navigation.right'
  | 'navigation.select'
  | 'navigation.back'
  | 'navigation.home';

interface MenuItem {
  id: 'youtube' | 'browser' | 'pc-mode' | 'settings';
  label: string;
  icon: string;
  kind: 'external-url' | 'system-command' | 'internal-page';
  target: string;
}

interface TvAppState {
  page: PageId;
  selectedId: MenuItem['id'];
  connection: ConnectionStatus;
  mode: DisplayMode;
  lastError?: string;
}
```

## 6. メニュー定義

メニューは`config/menu.ts`に定数として定義し、表示順とフォーカス順を一致させる。

| ID | 種別 | Target | 初期表示 |
|---|---|---|---|
| `youtube` | `external-url` | `https://www.youtube.com/` | 1行1列 |
| `browser` | `external-url` | ブラウザ開始ページ | 1行2列 |
| `pc-mode` | `system-command` | `system.switch_mode: pc` | 2行1列 |
| `settings` | `internal-page` | `settings` | 2行2列 |

外部URLはコード中に分散させず、設定ファイルへ集約する。許可するURLは`https`スキームに限定する。

## 7. フォーカス移動

### 7.1 座標

| ID | 行 | 列 |
|---|---:|---:|
| `youtube` | 0 | 0 |
| `browser` | 0 | 1 |
| `pc-mode` | 1 | 0 |
| `settings` | 1 | 1 |

### 7.2 移動規則

- 上下左右入力は、現在座標に方向差分を加えて移動先を検索する。
- グリッド外へ出る入力は無視し、現在位置を維持する。
- 非表示または無効な項目は候補から除外する。
- ホーム復帰時は`youtube`へフォーカスを戻す。
- マウスが項目へ入った場合は、その項目を選択状態にする。
- 連続入力はブラウザのキーリピートを許可するが、`select`は`keydown`の初回だけ処理する。

状態遷移は[draw.io](diagrams/tv-home.drawio)の「Navigation State」ページに定義する。

## 8. 入力マッピング

| 入力元 | 入力 | 内部アクション |
|---|---|---|
| キーボード | `ArrowUp` | `navigation.up` |
| キーボード | `ArrowDown` | `navigation.down` |
| キーボード | `ArrowLeft` | `navigation.left` |
| キーボード | `ArrowRight` | `navigation.right` |
| キーボード | `Enter` | `navigation.select` |
| キーボード | `Escape` | `navigation.back` |
| キーボード | `Home` | `navigation.home` |
| WebSocket | `navigation.*` | 同名アクション |

キーボードとWebSocketは同じReducerへ入力し、挙動差を作らない。

## 9. 画面遷移

| 現在画面 | 操作 | 遷移先または処理 |
|---|---|---|
| Home | YouTubeを決定 | YouTube URLを表示 |
| Home | Browserを決定 | ブラウザ開始URLを表示 |
| Home | PC Modeを決定 | CoreへPCモード切り替え要求を送信 |
| Home | Settingsを決定 | Settings |
| Settings | 戻る | Home |
| 任意の内部画面 | ホーム | Home、初期フォーカス |
| Home | 戻る | 状態変更なし |

外部サイトへ遷移するとReactのWebSocket接続は失われる。ホーム復帰はSuica CoreがChromiumをホームURLへ再表示することで実現する。

## 10. WebSocket接続

- 接続先は現在のページと同じホストの`/ws?role=tv`とする。
- 本番環境では`ws://127.0.0.1:3030/ws?role=tv`となる。
- 接続開始時の状態を`connecting`とする。
- 切断時は`reconnecting`とし、1秒、2秒、4秒、8秒、最大10秒で再試行する。
- 接続確立時に再試行回数をリセットする。
- JSON解析に失敗したメッセージは破棄し、診断ログを出力する。
- 未知の`type`または`action`は無視する。
- Reactのアンマウント時はタイマーとSocketを必ず破棄する。

## 11. UI仕様

| 項目 | 値 |
|---|---|
| 基準解像度 | 1920×1080 |
| セーフエリア | 画面端から左右96px、上下54px以上 |
| 最小本文サイズ | 28px |
| メニューラベル | 32px以上、太字 |
| フォーカス枠 | 4px以上、高コントラスト色 |
| アニメーション | 150ms以内。`prefers-reduced-motion`を尊重する。 |
| コントラスト | WCAG AA相当を目標とする。 |

画面が1280×720の場合もスクロールを発生させず、4項目が表示範囲へ収まること。

## 12. エラー処理

| 事象 | UI処理 |
|---|---|
| Core未接続 | 接続バッジを「再接続中」とし、PC Modeを無効化する。 |
| モード切り替え失敗 | ホーム画面を維持し、5秒間エラーメッセージを表示する。 |
| 不正メッセージ | 画面状態を変更せず、コンソールへ警告する。 |
| 外部URL設定不正 | 遷移せず、設定エラーを表示する。 |
| 外部サイト読込失敗 | Chromium標準エラーとなる。ホーム操作で復帰可能とする。 |

## 13. ログ

本番ログは接続、切断、再接続、受信したメッセージ種別、画面遷移失敗に限定する。ペアリングトークンとメッセージ本文全体は出力しない。

## 14. テスト設計

### 14.1 単体テスト

| ID | 対象 | 確認内容 |
|---|---|---|
| TV-U-01 | Reducer | 各方向で正しい項目へ移動する。 |
| TV-U-02 | Reducer | グリッド端でフォーカスを維持する。 |
| TV-U-03 | Reducer | Homeで画面とフォーカスを初期化する。 |
| TV-U-04 | Keyboard Hook | キーを正しい操作へ変換する。 |
| TV-U-05 | Core Client | 不正JSONで状態を変更しない。 |
| TV-U-06 | Launcher | 許可されないURLを拒否する。 |

### 14.2 コンポーネントテスト

| ID | 確認内容 |
|---|---|
| TV-C-01 | 選択項目だけにフォーカススタイルが付く。 |
| TV-C-02 | 未接続時にPC Modeが無効になる。 |
| TV-C-03 | Settingsから戻るとHomeへ遷移する。 |
| TV-C-04 | WebSocket操作とキーボード操作が同じ結果になる。 |

### 14.3 受入基準

- 1920×1080と1280×720で要素が欠けない。
- テレビから3m離れて選択項目を判別できる。
- キーボードだけで全メニューを選択できる。
- Core切断後に自動再接続し、操作を再開できる。
