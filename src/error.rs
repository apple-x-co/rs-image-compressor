use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompressorError {
    #[error("I/Oエラー: {0}")]
    IoError(#[from] std::io::Error),

    // #[error("画像フォーマットエラー: {0}")]
    // ImageFormatError(String),

    #[error("画像デコードエラー: {0}")]
    ImageDecodeError(#[from] image::ImageError),

    #[error("PNG最適化エラー: {0}")]
    PngOptimizeError(String),

    #[error("JPEG圧縮エラー: {0}")]
    JpegCompressError(String),

    #[error("WebP圧縮エラー: {0}")]
    WebpCompressError(String),

    #[error("GIF圧縮エラー: {0}")]
    GifCompressError(String),

    #[error("HEIF圧縮エラー: {0}")]
    HeifCompressError(String),

    #[error("PDF圧縮エラー: {0}")]
    PdfCompressError(String),

    #[error("SVG圧縮エラー: {0}")]
    SvgCompressError(String),

    #[error("設定エラー: {0}")]
    ConfigError(String),

    #[error("不明なファイル形式")]
    UnknownFileFormat,

    #[error("JSON形式エラー: {0}")]
    JsonUnformatError(String),
}

// pub type Result<T> = std::result::Result<T, CompressorError>;