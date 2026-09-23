# Suica TV Raspberry Piセットアップ手順

この手順では、Raspberry Pi OS 64-bit DesktopへSuica TVをcloneし、React画面とSuica Coreをビルドして、ログイン時に自動起動させる。

## 1. 前提

| 項目 | 条件 |
|---|---|
| 本体 | Raspberry Pi 5（8GB推奨） |
| OS | Raspberry Pi OS 64-bit Desktop |
| Node.js | 18以上 |
| Rust | rustupのstable toolchain |
| ブラウザ | Chromium |

Suica Coreはユーザーサービスとして動く。デスクトップへ自動ログインする利用者で以下の操作を行う。rootでは実行しない。

## 2. OSと必要パッケージを準備する

```bash
sudo apt update
sudo apt full-upgrade -y
sudo apt install -y git curl build-essential pkg-config nodejs npm chromium wtype xdotool
```

必要ならカーソルを隠すために`unclutter`を追加する。

```bash
sudo apt install -y unclutter
```

Node.jsが18未満なら18以上へ更新してから続行する。

## 3. Rust stableをインストールする

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup default stable
rustc --version
cargo --version
```

## 4. GitHubからcloneする

`<GitHubリポジトリURL>`を実際のURLへ置き換える。

```bash
cd "$HOME"
git clone <GitHubリポジトリURL> suica-tv
cd "$HOME/suica-tv"
```

秘密の値やアクセストークンはリポジトリへ保存しない。

## 5. ReactとCoreをインストールする

付属スクリプトは、ReactとRustのテスト・リリースビルド、バイナリとsystemdユーザーサービスの配置をまとめて行う。

```bash
cd "$HOME/suica-tv"
chmod +x scripts/install-raspberry-pi.sh
./scripts/install-raspberry-pi.sh
```

手動で検証する場合:

```bash
cd "$HOME/suica-tv/apps/tv"
npm ci
npm test
npm run build

cd "$HOME/suica-tv"
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release --workspace
```

## 6. Coreを設定する

設定ファイルは`~/.config/suica-tv/suica-core.toml`に作成される。必ず`pairing_code`を変更する。

6桁コードの生成例:

```bash
printf '%06d\n' "$(( $(od -An -N4 -tu4 /dev/urandom) % 1000000 ))"
```

```bash
nano "$HOME/.config/suica-tv/suica-core.toml"
chmod 600 "$HOME/.config/suica-tv/suica-core.toml"
```

最低限、次を確認する。`pi`は実際のユーザー名へ置き換える。

```toml
bind_address = "0.0.0.0"
port = 3030
static_dir = "/home/pi/suica-tv/apps/tv/dist"
home_url = "http://127.0.0.1:3030/"
chromium_binary = "/usr/bin/chromium"
profile_dir = "/home/pi/.local/share/suica-tv/chromium-profile"
pid_file = "/home/pi/.local/share/suica-tv/suica-tv.pid"
token_store = "/home/pi/.local/share/suica-tv/tokens.json"
pairing_code = "CHANGE_ME"
system_backend = "auto"
input_backend = "auto"
wtype_binary = "/usr/bin/wtype"
xdotool_binary = "/usr/bin/xdotool"
```

`CHANGE_ME`のままではCoreは起動しない。設定値は環境変数でも上書きできる。例: `SUICA_CORE_PAIRING_CODE=123456`。コード、発行済みトークン、AuthorizationヘッダーをログやGitへ残さない。

## 7. systemdユーザーサービスを有効にする

```bash
systemctl --user daemon-reload
systemctl --user enable --now suica-core
systemctl --user status suica-core
```

ログを確認する。

```bash
journalctl --user -u suica-core -f
```

ログアウト後もサービスを維持する必要がある場合だけlingerを有効にする。

```bash
sudo loginctl enable-linger "$USER"
```

## 8. 動作確認

```bash
curl http://127.0.0.1:3030/health/live
curl http://127.0.0.1:3030/health/ready
```

ペアリングを確認する。コードは実設定の値へ置き換える。

```bash
curl -X POST http://127.0.0.1:3030/api/v1/pair \
  -H 'Content-Type: application/json' \
  -d '{"code":"123456","deviceName":"My iPhone"}'
```

レスポンスの`token`は一度しか表示されない。iPhoneのKeychainへ保存し、平文ファイルには保存しない。Core側はSHA-256ハッシュだけを`tokens.json`へ記録する。

テレビ画面は`http://127.0.0.1:3030/`で開く。

React開発時のTVロール接続先:

```bash
VITE_CORE_WS_URL='ws://127.0.0.1:3030/ws?role=tv&protocolVersion=1' npm run dev
```

本番ではCoreがReactの`dist`を配信し、同じホストのWebSocketへ接続する。

## 9. 更新

```bash
cd "$HOME/suica-tv"
git pull --ff-only
./scripts/install-raspberry-pi.sh
systemctl --user restart suica-core
```

既存の設定とトークンDBは更新時も保持される。

## 10. トラブルシューティング

### Coreが起動しない

```bash
systemctl --user status suica-core
journalctl --user -u suica-core -n 100 --no-pager
```

`pairing_code must be changed`の場合は`CHANGE_ME`を非公開の6桁へ変更する。Reactの`dist`がない場合は`npm run build`を再実行する。

### Chromiumが見つからない

```bash
command -v chromium
command -v chromium-browser
```

実際のコマンドを`chromium_binary`へ設定する。

### PCモードへ切り替わらない

```bash
cat "$HOME/.local/share/suica-tv/suica-tv.pid"
journalctl --user -u suica-core -n 100 --no-pager
```

Coreは専用`--user-data-dir`を持つChromiumだけを終了し、通常利用中のChromiumには干渉しない。

### LAN上のiPhoneから接続できない

Raspberry PiのIP、iPhoneと同じ家庭内ネットワークであること、`bind_address = "0.0.0.0"`を確認する。

```bash
hostname -I
```

家庭内LAN外へ3030番ポートを公開しない。v0.1はHTTPS/WSSを含まない。

### YouTube表示中にリモコン操作できない

CoreはWaylandでは`wtype`、X11では`xdotool`を使い、Reactが外部ページへ遷移した後の
十字キー・決定・戻る・文字入力をChromiumへ送る。次を確認する。

```bash
command -v wtype
command -v xdotool
systemctl --user show-environment | grep -E 'WAYLAND_DISPLAY|DISPLAY|XDG_RUNTIME_DIR'
```

画面セッションの環境変数が表示されない場合は、そのセッション内で次を実行してCoreを再起動する。

```bash
systemctl --user import-environment WAYLAND_DISPLAY DISPLAY XDG_RUNTIME_DIR
systemctl --user restart suica-core
```
