# 回帰試験

## 自動試験

変更をpushする前に、リポジトリルートで次を実行する。

```bash
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
(cd apps/tv && npm test && npm run typecheck && npm run build)
xcodebuild -project apps/ios/SuicaRemote.xcodeproj \
  -scheme SuicaRemote \
  -destination 'platform=iOS Simulator,name=iPhone 15' test
```

自動試験で次を確認する。

| 経路 | 確認内容 |
|---|---|
| iOS | 切断後の再接続、再接続要求の競合、`server.hello`後の接続確定、コマンド生成 |
| Core | Remote→TV転送、PC切替、ホーム復帰、TV再接続、重複リクエスト、タイムアウト復旧 |
| React | iOSのOK操作によるタイル起動、Core再起動後の再接続、不正payloadの拒否 |

## Raspberry Pi実機試験

実機試験の前に環境を記録する。

```bash
cat /etc/os-release
uname -a
chromium --version || chromium-browser --version
echo "XDG_SESSION_TYPE=$XDG_SESSION_TYPE"
echo "WAYLAND_DISPLAY=$WAYLAND_DISPLAY"
echo "DISPLAY=$DISPLAY"
systemctl --user status suica-core --no-pager
```

以下は自動試験の成功だけでは完了扱いにしない。

1. ホームでiPhoneのOKを押し、YouTubeを開く。
2. 文字入力、上下スクロール、再生、戻るを確認する。
3. ホームへ戻り、ChromiumとTV WebSocketが1つだけで操作可能なことを確認する。
4. PCモードからホームへ戻り、Chromiumが二重起動しないことを確認する。
5. iPhoneのWi-Fiを切断・復旧し、10秒以内に再接続を試行することを確認する。
6. `systemctl --user restart suica-core` 後にiOSとReactが再接続することを確認する。
7. 失敗時は時刻、操作、リクエストID、接続状態、Chromium PIDを記録する。トークンや認証ヘッダーは記録しない。
