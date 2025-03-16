use anyhow::{anyhow, Result};
use oxipng::{Options, PngError};
use std::fs::File;
use std::io::{BufReader, Read};

// NOTE: Use "Oxipng"
// TODO: -o 値は最適化レベルを表し、1～6,maxの順に圧縮レベルが高くなります(デフォルトは2)。
// TODO: -i 0 or 1はインターレースの有無を表し、0で削除、1で有効化し、アルゴリズムはAdam7PNGを使用します。
// TODO: --strip safe or allは画像のメタデータを削除する設定です。safeは画像の描画に影響しないメタデータを削除します(-sでも同じ設定)。
// TODO: allは全てのメタデータを削除します。
// TODO: ちなみに--zopfliを加えることで、zopfliのアルゴリズムを用いてより効果的な圧縮もできます(ただし処理はかなり遅いのでリアルタイム処理には向いていない)。

pub fn png_compressor(input_file: &mut File) -> Result<Vec<u8>> {
    let mut reader = BufReader::new(input_file);
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;

    let png_result = oxipng::optimize_from_memory(&bytes, &Options::default());
    match png_result {
        Ok(data) => Ok(data),
        Err(e) => {
            match e {
                PngError::DeflatedDataTooLong(size) => {
                    Err(anyhow!("Deflated data too long: {}", size))
                }
                PngError::TimedOut => {
                    Err(anyhow!("PNG optimization timed out"))
                }
                PngError::NotPNG => {
                    Err(anyhow!("Invalid PNG header: Not a PNG file or file is corrupted"))
                }
                PngError::APNGNotSupported => {
                    Err(anyhow!("APNG format is not supported"))
                }
                PngError::APNGOutOfOrder => {
                    Err(anyhow!("APNG chunks are out of order"))
                }
                PngError::InvalidData => {
                    Err(anyhow!("Invalid PNG data"))
                }
                PngError::TruncatedData => {
                    Err(anyhow!("Truncated PNG data"))
                }
                PngError::ChunkMissing(chunk_type) => {
                    Err(anyhow!("Missing PNG chunk: {}", chunk_type))
                }
                PngError::InvalidDepthForType(bit_depth, color_type) => {
                    Err(anyhow!("Invalid bit depth for color type: bit_depth={:?}, color_type={:?}", bit_depth, color_type))
                }
                PngError::IncorrectDataLength(expected, actual) => {
                    Err(anyhow!("Incorrect data length: expected={}, actual={}", expected, actual))
                }
                PngError::C2PAMetadataPreventsChanges => {
                    Err(anyhow!("C2PA metadata prevents changes"))
                }
                PngError::Other(message) => {
                    Err(anyhow!("PNG optimization failed: {}", message))
                }
                _ => {
                    Err(anyhow!("PNG optimization failed: {:?}", e))
                }
            }
        }
    }
}