# Chromium OS連携 詳細設計書

## 1. 目的と範囲

本書は、Raspberry Pi OS上でSuica CoreとSuica TV Homeを自動起動し、Chromiumキオスクと通常デスクトップを安全に切り替えるためのOS連携を定義する。

対象はsystemd、デスクトップセッション、Chromium専用プロファイル、起動スクリプト、PID管理、ログ、権限および復旧である。

## 2. 配置構成

```text
/opt/suica-tv/
├── bin/
│   └── suica-core
├── tv/
│   ├── index.html
│   └── assets/
└── scripts/
    ├── start-tv.sh
    ├── start-pc.sh
    └── health-check.sh

/etc/suica-tv/
└── suica-core.toml

/var/lib/suica-tv/
├── chromium-profile/
├── suica-tv.pid
└── tokens.json

/var/log/
└── journal/          journaldで管理

~/.config/systemd/user/
├── suica-core.service
└── suica-tv-kiosk.service
```

## 3. 実行ユーザーと権限

- Suica CoreとChromiumはデスクトップへログインする一般ユーザーで実行する。
- root権限およびsudoを要求しない。
- 設定ファイルは所有者だけが書き込めるようにする。
- トークン保存ファイルはパーミッション600とする。
- Coreバイナリ、静的ファイル、スクリプトは一般ユーザーから書き換え不可とする運用を推奨する。
- 任意のコマンド実行を防ぐため、Coreはシェル経由でChromiumを起動しない。

## 4. systemdユニット

### 4.1 suica-core.service

```ini
[Unit]
Description=Suica TV Core
After=network-online.target graphical-session.target
Wants=network-online.target

[Service]
Type=notify
ExecStart=/opt/suica-tv/bin/suica-core --config /etc/suica-tv/suica-core.toml
Restart=on-failure
RestartSec=2
TimeoutStopSec=15
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ReadWritePaths=/var/lib/suica-tv

[Install]
WantedBy=default.target
```

Coreが`sd_notify`に対応できない段階では`Type=simple`を使用する。`READY=1`はListen開始と初期状態判定の完了後に送信する。

### 4.2 suica-tv-kiosk.service

```ini
[Unit]
Description=Suica TV Kiosk
After=suica-core.service graphical-session.target
Requires=suica-core.service

[Service]
Type=simple
ExecStart=/opt/suica-tv/scripts/start-tv.sh
ExecStop=/opt/suica-tv/scripts/start-pc.sh
Restart=on-failure
RestartSec=3
Environment=DISPLAY=:0

[Install]
WantedBy=default.target
```

Wayland環境では`DISPLAY`固定に依存せず、デスクトップセッションから継承される`WAYLAND_DISPLAY`と`XDG_RUNTIME_DIR`を使用する。実機OSのデフォルトセッションを確認してユニットを確定する。

## 5. Chromium起動仕様

### 5.1 起動引数

```text
/usr/bin/chromium
  --kiosk
  --no-first-run
  --disable-session-crashed-bubble
  --disable-infobars
  --autoplay-policy=no-user-gesture-required
  --user-data-dir=/var/lib/suica-tv/chromium-profile
  --app=http://127.0.0.1:3030/
```

| 引数 | 目的 |
|---|---|
| `--kiosk` | 全画面キオスク表示 |
| `--no-first-run` | 初回起動ダイアログを抑止 |
| `--disable-session-crashed-bubble` | 異常終了後の復元通知を抑止 |
| `--autoplay-policy` | テレビ向け動画再生を許可 |
| `--user-data-dir` | Suica TV専用プロファイルを分離 |
| `--app` | アドレスバーを表示せずホームを開く |

Chromiumのコマンド名が`chromium-browser`であるOSイメージでは、設定ファイルの`chromium_binary`を変更する。

### 5.2 起動成功判定

次をすべて満たした場合に成功とする。

1. spawnが成功してPIDを取得できる。
2. 500ミリ秒後にプロセスが生存している。
3. `http://127.0.0.1:3030/health/ready`が200を返す。
4. 10秒以内に上記条件が成立する。

画面ピクセルの内容まではv0.1で自動判定しない。実機統合試験でHDMI表示を確認する。

## 6. PID管理

- Chromium起動直後に一時ファイルへPIDを書き、fsync後に`/var/lib/suica-tv/suica-tv.pid`へ置換する。
- 終了前にPIDが存在し、`/proc/{pid}/cmdline`に専用`user-data-dir`が含まれることを確認する。
- 条件が一致しないPIDへシグナルを送信しない。
- プロセス終了確認後にPIDファイルを削除する。
- PIDファイルだけが残っている場合はstaleとして削除し、起動を継続する。

## 7. start-tv処理

1. CoreのReadyエンドポイントを最大10秒待つ。
2. PIDファイルを検証する。
3. 既に正しいChromiumが生存している場合は成功終了する。
4. stale PIDファイルを削除する。
5. 固定引数でChromiumを起動する。
6. PIDを安全に保存する。
7. 生存確認を実行する。
8. カーソル非表示を有効にする。

スクリプトを使用する場合も、外部入力をコマンド文字列へ連結しない。可能な限り最終実装はRustの`ChromiumController`へ集約する。

## 8. start-pc処理

1. PIDファイルを読み込む。
2. PIDとコマンドラインを検証する。
3. 管理対象が存在しない場合は冪等な成功とする。
4. SIGTERMを送信する。
5. 最大5秒、250ミリ秒間隔で終了を確認する。
6. 終了しない場合はSIGKILLを送信する。
7. 最大1秒、終了を確認する。
8. PIDファイルを削除する。
9. カーソル非表示を解除する。

通常のデスクトップでユーザーが開いたChromiumは専用プロファイルではないため、終了対象にしない。

## 9. カーソル制御

TVモードでは`unclutter`またはデスクトップ環境のAPIを使用し、一定時間後にカーソルを非表示にする。PCモードでは非表示プロセスを停止する。

| モード | 動作 |
|---|---|
| TV | 1秒間入力がなければ非表示。マウス移動時も短時間だけ表示。 |
| PC | 常に通常表示。 |

## 10. 起動順序

1. Raspberry Pi OSが起動する。
2. 自動ログインでグラフィカルセッションが開始する。
3. `suica-core.service`が開始する。
4. Coreが静的配信とWebSocket待受を開始する。
5. `suica-tv-kiosk.service`がCoreのReadyを確認する。
6. Chromiumキオスクを起動する。
7. Suica TV HomeがCoreへWebSocket接続する。

起動構成図は[draw.io](diagrams/system-integration.drawio)の「Boot Dependencies」ページに定義する。

## 11. 復旧設計

| 障害 | 検知 | 復旧 |
|---|---|---|
| Core異常終了 | systemd | 2秒後に再起動 |
| Chromium異常終了 | Coreが2秒間隔で監視 | 即時、その後1、2、4、8秒後に再起動を試行 |
| React配信失敗 | Readyが503 | Chromium起動を待機 |
| Wi-Fi未接続 | network-online待機のタイムアウト | LAN外機能なしで起動継続、接続後利用可能 |
| デスクトップ未起動 | 環境変数またはdisplay接続失敗 | kiosk unitを再試行 |
| 起動ループ | systemd StartLimit | 停止してログを保持 |

`StartLimitBurst=5`、`StartLimitIntervalSec=60`を基準とし、短時間に5回失敗した場合は自動再起動を止める。

Coreの監視はTVモードが安定状態のときだけChromiumを復旧する。PCモード中や別のモード遷移中は起動しない。再起動は最大5回で止め、全試行が失敗した場合は実際のPC表示を状態として保持し、接続中のクライアントへ通知する。

## 12. ログ

- Coreログとスクリプト出力はjournaldへ集約する。
- `journalctl --user -u suica-core.service`で確認できるようにする。
- Chromiumの標準出力は`journalctl --user -u suica-tv-kiosk.service`で確認できるようにする。
- ローテーションはjournald設定へ従う。
- トークン、Cookie、閲覧URLのクエリ文字列はログへ出力しない。

## 13. 更新とロールバック

1. 新バージョンを別ディレクトリへ展開する。
2. React静的ファイルとCoreバイナリの整合する組を配置する。
3. シンボリックリンクを新バージョンへ切り替える。
4. Coreを再起動する。
5. ReadyとTVホーム表示を確認する。
6. 失敗時はリンクを旧バージョンへ戻す。

v0.1で自動更新機能は実装しないが、手動更新でロールバックできる構造とする。

## 14. 試験設計

| ID | 確認内容 |
|---|---|
| OS-I-01 | OS起動後にCoreとChromiumが自動起動する。 |
| OS-I-02 | PCモードで専用Chromiumだけが終了する。 |
| OS-I-03 | TV復帰時にChromiumが1プロセスだけ起動する。 |
| OS-I-04 | stale PIDファイルがあっても復旧する。 |
| OS-I-05 | Core異常終了後にsystemdが再起動する。 |
| OS-I-06 | Chromium異常終了後にTV表示へ復旧する。 |
| OS-I-07 | 5回連続失敗後に起動ループを停止する。 |
| OS-I-08 | 一般ユーザーのChromiumを終了しない。 |
| OS-I-09 | HDMI切断と再接続後に表示が復帰する。 |

## 15. 受入基準

- 電源投入後、ユーザー操作なしでTVホームが表示される。
- TVからPC、PCからTVを10回繰り返してプロセスが増殖しない。
- Raspberry Pi再起動後にTVモードへ復旧する。
- 通常デスクトップのユーザープロセスを誤終了しない。
- システム制御にroot権限を必要としない。
