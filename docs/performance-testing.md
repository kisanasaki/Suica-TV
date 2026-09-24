# Raspberry Pi 性能・長時間稼働試験

Issue #27の測定手順。測定結果は機種、OS、解像度、冷却、ブラウザ、ネットワーク条件と組にして扱い、未測定構成の対応を保証しない。

## 事前記録

- Raspberry Piのモデルとメモリ容量（基準: Pi 5 8GB、比較: Pi 4 2GB）
- Raspberry Pi OS、カーネル、Chromium、Suica Coreのバージョン
- 画面解像度、Wayland/X11、ハードウェアアクセラレーション、冷却、ストレージ
- 有線／無線LAN、動画の解像度、同時実行機能

## 収集

CoreとChromiumを通常構成で起動し、リポジトリのルートから実行する。

```bash
chmod +x scripts/benchmark-pi.sh
./scripts/benchmark-pi.sh \
  --duration-seconds 3600 \
  --interval-seconds 5 \
  --output-dir "$HOME/suica-benchmark/idle-$(date +%Y%m%d-%H%M%S)"
```

`metadata.txt`に環境、`samples.csv`に温度・負荷・空きメモリ・Core/Chromium RSS・ヘルスAPI応答時間、`summary.json`に実行条件を保存する。センサーや対象プロセスがない項目は空欄になり、推測値で埋めない。

## シナリオ

同じ条件と収集時間で次を個別に測定する。

1. ホーム画面のアイドル状態。
2. YouTube 1080p動画の連続再生。再生統計からドロップフレームと音ずれを別途記録する。
3. YouTubeからホームへ100回復帰し、失敗回数と操作応答を記録する。
4. 8時間以上の通常利用。RSSの継続増加、Core/Chromium再起動、操作不能を確認する。
5. 将来機能は写真取込、録音、AI処理、UxPlayを一つずつ追加し、基準測定と分離する。

## 判定

- 通常操作の目標は500ms以内、表示モード切替は10秒以内とする。
- 温度スロットリング、OOM、Core/Chromiumの意図しない再起動、ホーム復帰不能は失敗として記録する。
- Pi 4 2GBの対応範囲はPi 5と同じシナリオを完走した実測結果から決める。
- 実機でのみ確認できる動画品質、CEC、AirPlay、音声出力を自動収集結果だけで合格にしない。
