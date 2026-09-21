# Suica TV

Raspberry Pi 5を家庭用テレビへ接続し、iPhoneから操作するテレビ用PCプロジェクトです。

## ドキュメント

- [Suica TV v0.1 基本設計書](docs/design.md)
- [設計図 draw.io](docs/diagrams/suica-tv-v0.1.drawio)
- [モジュール別 詳細設計書](docs/detailed-design/README.md)
- [Raspberry Piセットアップ手順](docs/setup-raspberry-pi.md)
- [Remote API契約](contracts/remote-api.md)

## Reactテレビ画面

```bash
cd apps/tv
npm install
npm run dev
```

開発画面は`http://localhost:5173`で表示されます。Suica Coreの接続先を変更する場合は、`.env.example`を参考に`VITE_CORE_WS_URL`を設定してください。Coreが未起動でもホーム画面とキーボード操作は確認できます。

```bash
npm test
npm run build
```

## Suica Core

Suica CoreはReact静的ファイルの配信、iPhone/TV WebSocket接続、ペアリング、表示モードとChromiumの制御を担当します。

```bash
# 先に apps/tv をビルドする
cd apps/tv
npm ci
npm run build
cd ../..

# 実設定を作成し、pairing_codeを非公開の6桁へ変更する
cp config/suica-core.example.toml config/suica-core.toml

cargo run -p suica-core -- --config config/suica-core.toml
```

起動後は`http://127.0.0.1:3030/`でホーム画面、`/health/live`と`/health/ready`で稼働状態を確認できます。Windowsではシステム操作を自動的にシミュレーションし、Raspberry Pi OSでは専用Chromiumを実際に制御します。

APIと運用の詳細は[Raspberry Piセットアップ手順](docs/setup-raspberry-pi.md)および[WebSocket API詳細設計](docs/detailed-design/05-websocket-api.md)を参照してください。

## iPhoneリモコン

`apps/ios/SuicaRemote.xcodeproj`をXcode 15.1以降で開き、iOS 17以降のSimulatorまたはiPhoneで`SuicaRemote`スキームを実行します。

実機で動かす場合は、Xcodeの`Signing & Capabilities`で自身のDevelopment Teamを選択してください。初回起動後、設定画面でRaspberry Piのホスト名とポートを確認し、Coreに設定した6桁のペアリングコードを入力します。ローカルネットワークへのアクセス確認が表示されたら許可してください。

```bash
xcodebuild test \
  -project apps/ios/SuicaRemote.xcodeproj \
  -scheme SuicaRemote \
  -destination 'platform=iOS Simulator,name=iPhone 15'
```
