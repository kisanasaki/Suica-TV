# UxPlay実機検証手順

Issue #28の第1段階。Raspberry Pi 5上でAirPlay画面ミラーリングの実現性を確認してからSuica Coreへ統合する。Netflix等の保護コンテンツ回避、録画、複数端末の同時表示は対象外。

## 1. 環境確認

```bash
apt-cache policy uxplay
uxplay -h
./scripts/check-uxplay.sh
```

OS、iOS、UxPlay、GStreamer、Wayland/X11、解像度、音声出力先を記録する。パッケージ導入が必要な場合は、対象OSで候補を確認してから次を個別にインストールする。

```bash
sudo apt install uxplay \
  gstreamer1.0-plugins-base gstreamer1.0-libav \
  gstreamer1.0-plugins-good gstreamer1.0-plugins-bad
```

環境により`gstreamer1.0-gl`、`gstreamer1.0-gtk3`、`gstreamer1.0-x`、PipeWire/PulseAudio/ALSA用プラグインが追加で必要になる。診断結果にないパッケージを推測で追加しない。

## 2. 単体起動

最初は録画オプションを付けず、受信名、ホスト名非表示、全画面、固定ポート、ランダムPINを指定する。

```bash
uxplay -n "Suica TV" -nh -fs -p 35000 -pin
```

- UDP 5353のmDNSと、TCP/UDP 35000〜35002を家庭内LANで到達可能にする。
- 自動映像出力が失敗する場合だけ、Waylandは`-vs waylandsink`、デスクトップOpenGLは`-vs glimagesink`、Lite/KMSは`-vs kmssink`を一つずつ試す。
- PINなし運用へ緩めず、採用する認証方法を実機結果とともに決める。
- Raspberry Pi 5では旧Pi向けのBroadcom H.264オプションを前提にしない。

## 3. 確認シナリオ

1. iPhoneとPiを同じ家庭内LANへ接続し、「画面ミラーリング」に`Suica TV`が現れることを確認する。
2. PIN認証後、ホーム画面、写真、保護されていない動画とHDMI音声を確認する。
3. 縦横回転、停止・再接続、Wi-Fi断、iPhoneスリープ、UxPlay強制終了を確認する。
4. Chromium起動中と停止中を分け、映像・音声・フォーカスの競合を記録する。
5. `scripts/benchmark-pi.sh`を併用してCPU、RSS、温度、遅延、音ずれを記録する。

## 4. Core統合へ進む条件

- 安定する映像sink・音声sink・必要パッケージが確定している。
- UxPlayの起動成功、受信中、切断、異常終了を判定できるログまたは状態取得方法が確認できる。
- Chromiumとの排他方法と、停止後にReactホームへ確実に戻る手順が確認できる。
- 採用するPIN/登録方式と家庭内LANのポート設定が確定している。

条件が揃うまでは、Coreの状態を「受信中」と推測表示したり、UxPlayを常駐サービスとして自動有効化したりしない。
