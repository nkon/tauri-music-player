# macOS向けビルド手順

## 1. 開発環境の確認

必要なツール:
- Xcode と Command Line Tools
- Node.js
- pnpm
- Rust (rustup経由)

```bash
# Xcodeセットアップ確認
xcode-select --print-path
sudo xcodebuild -license accept

# Rustツールチェーン確認
rustup show
```

## 2. Rustターゲットの追加

```bash
# Apple Silicon (M1/M2) と Intel Mac向けのターゲットを追加
rustup target add aarch64-apple-darwin x86_64-apple-darwin
```

## 3. ビルド手順

```bash
# プロジェクトルートで依存関係をインストール
pnpm install

# フロントエンドをビルド
pnpm build

# Tauriアプリをビルド
cd src-tauri

# Apple Silicon (M1/M2) 向け
cargo tauri build --target aarch64-apple-darwin

# Intel Mac向け
cargo tauri build --target x86_64-apple-darwin
```

## 4. ビルド成果物

ビルドされたアプリケーションは以下のディレクトリに生成されます:
- `src-tauri/target/aarch64-apple-darwin/release/bundle/macos/` (Apple Silicon向け)
- `src-tauri/target/x86_64-apple-darwin/release/bundle/macos/` (Intel向け)

## 5. iOS向けビルド手順の詳細

事前準備:
```bash
# 1. 開発チームIDの設定
# Apple Developer サイトで確認したチームIDを設定
export TAURI_APPLE_DEVELOPMENT_TEAM="YOUR_TEAM_ID"

# 2. プロジェクトの再生成
rm -rf src-tauri/gen/apple
pnpm tauri ios init

# 3. Xcodeプロジェクトを開く
open src-tauri/gen/apple/tauri-music-player.xcodeproj
```

Xcodeでの設定:
1. プロジェクトナビゲータでtauri-music-player_iOSターゲットを選択
2. Signing & Capabilitiesタブを選択
3. Team欄で開発チームを選択
4. 「Automatically manage signing」にチェック

設定後:
```bash
pnpm tauri ios build
```

### Info.plist の確認項目

src-tauri/gen/apple/tauri-music-player_iOS/Info.plist に以下が設定されていることを確認:

```xml
<key>LSApplicationCategoryType</key>
<string>public.app-category.music</string>
<key>UIBackgroundModes</key>
<array>
    <string>audio</string>
</array>
<key>LSSupportsOpeningDocumentsInPlace</key>
<true/>
<key>UIFileSharingEnabled</key>
<true/>
<key>NSAppTransportSecurity</key>
<dict>
    <key>NSAllowsArbitraryLoads</key>
    <true/>
</dict>
```

### ビルドエラーのトラブルシューティング

1. "Missing iOS distribution signing certificate"
```bash
# 証明書の確認
security find-identity -v -p codesigning

# 証明書のインポート
security import distribution.p12 -k ~/Library/Keychains/login.keychain-db
```

2. "Failed to create provisioning profile"
- Apple Developer ポータルで以下を確認:
  * App ID の登録
  * Provisioning Profile の作成
  * Device の登録（実機テスト用）

3. "Code signing is required for product type 'Application'"
- Xcode で自動署名を有効化:
  * TARGETS → Signing & Capabilities
  * "Automatically manage signing" をチェック

### project.yml の設定

src-tauri/gen/apple/project.yml が存在することを確認:

```bash
# project.ymlのパーミッション確認
ls -l src-tauri/gen/apple/project.yml

# パーミッションの修正（必要な場合）
chmod 644 src-tauri/gen/apple/project.yml
```

エラーが続く場合:
```bash
# プロジェクトの再生成
rm -rf src-tauri/gen/apple
pnpm tauri ios init
```
