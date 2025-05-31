# rs-image-compressor

⚡ **高速で軽量な画像・文書圧縮ツール** - Rust製のマルチフォーマット対応コマンドラインアプリケーション

## 🚀 概要

`rs-image-compressor`は、様々な画像・文書ファイルを効率的に圧縮するRust製のコマンドラインツールです。高品質な圧縮アルゴリズムと豊富な設定オプションで、ファイルサイズを大幅に削減しながら品質を保持します。

## 📦 対応フォーマット

- **画像**: PNG, JPEG, WebP, GIF, HEIF/HEIC
- **文書**: PDF, SVG/XML

## ✨ 主な特徴

- 🔧 **高度な設定**: JSONスキーマによる詳細な圧縮パラメータ設定
- 🎯 **品質重視**: lossy/lossless圧縮の選択、品質レベルの細かい調整
- 📏 **リサイズ機能**: 複数のフィルタアルゴリズム対応（Lanczos3, CatmullRom等）
- 🖼️ **EXIF処理**: JPEG画像の自動回転、メタデータ保持/削除の選択
- 📚 **PDF最適化**: 画像圧縮、フォント削除、メタデータクリーンアップ
- 🎬 **GIF最適化**: フレーム最適化、ループ設定、品質調整
- ⚡ **高速処理**: Rustの並列処理とメモリ効率性を活用
- 🛡️ **安全性**: 型安全性とエラーハンドリングによる信頼性の高い処理

## 🛠️ 技術スタック

- **言語**: Rust 2024 Edition
- **圧縮ライブラリ**: oxipng, mozjpeg, webp, gifski, libheif-rs
- **画像処理**: image crate, imagequant
- **PDF処理**: lopdf, lcms2（カラープロファイル変換）
- **設定**: JSON Schema バリデーション

## 📖 使用例

```bash
# 基本的な圧縮
rs-image-compressor -i input.jpg -o output.jpg

# 設定ファイルを使用した詳細圧縮
rs-image-compressor -i input.png -o output.png -c config.json

# 詳細ログ付き
rs-image-compressor -i input.pdf -o output.pdf -v
```

## 🎯 対象ユーザー

- Web開発者（画像最適化）
- デザイナー（ファイルサイズ削減）
- システム管理者（バッチ処理）
- 文書管理（PDF最適化）

---

**軽量・高速・設定豊富** - あらゆる圧縮ニーズに対応する、次世代の画像・文書圧縮ツール