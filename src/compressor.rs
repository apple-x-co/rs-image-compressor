mod jpeg_compressor;
mod png_compressor;

use crate::config_json::Config;
use anyhow::{Context, Result, anyhow};
use image::ImageFormat;
use image::ImageReader;
use little_exif::exif_tag::ExifTag;
use little_exif::filetype::FileExtension;
use little_exif::metadata::Metadata;
use std::fs::File;
use std::io::{BufReader, Read, Seek, Write};
use std::path::Path;
use std::time::Instant;

// TODO: できるだけ以下情報はこのプログラム側で表示する。各画像毎の圧縮プログラムでは出力しない。
// [INFO] 圧縮開始: example_image.jpg
// [INFO] 入力ファイル:
//     - ファイル名: example_image.jpg
//     - ファイルサイズ: 2.5 MB
//     - 画像解像度: 1920x1080
//     - 画像形式: JPEG
//
// [INFO] 使用する圧縮アルゴリズム: JPEG（品質設定: 85）
// [INFO] 圧縮前のサイズ: 2.5 MB
// [INFO] 圧縮後のサイズ: 1.2 MB
// [INFO] 圧縮率: 52%
//
// [INFO] 圧縮処理中...
// [INFO] 圧縮完了: example_image_compressed.jpg
//     - 出力ファイル: example_image_compressed.jpg
//     - 出力ファイルサイズ: 1.2 MB
//     - 出力形式: JPEG
//
// [INFO] 処理時間: 2.3秒
//
// 圧縮完了: example_image.jpg

pub fn compress(
    config: Config,
    verbose: bool,
    input_path: &String,
    output_path: &String,
) -> Result<()> {
    let now = Instant::now();
    let input_file_name = Path::new(input_path)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let output_file_name = Path::new(output_path)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();

    if verbose {
        println!("Start");
        println!("Input:");
        println!("\tFile name: {:?}", input_file_name);
        // println!("\tSize: {:?} bytes", 1000); // FIXME:
        // println!("\tResolution: {:?} x {:?}", 8000, 600); // FIXME:
        // println!("\tFormat: JPEG"); // FIXME:
    }

    let input_file = File::open(input_path)
        .with_context(|| format!("Failed to open input file: {}", input_path))?;

    let reader = BufReader::new(input_file);
    let image_reader = ImageReader::new(reader)
        .with_guessed_format()
        .context("Failed to guess image format")?;

    let image_format = match image_reader.format() {
        Some(format) => format,
        None => return Err(anyhow::anyhow!("Could not determine image format")),
    };

    // NOTE: Compress image
    let compressed_data = match image_format {
        ImageFormat::Png => {
            let mut input_file = File::open(input_path)
                .with_context(|| format!("Failed to open input file: {}", input_path))?;
            let result = png_compressor::compress(config.png.as_ref(), &mut input_file);
            match result {
                Ok(data) => data,
                Err(e) => {
                    return Err(anyhow!(
                        "PNG compression failed for file: {}. Error: {}",
                        input_path,
                        e
                    ));
                }
            }
        }
        ImageFormat::Jpeg => {
            let mut input_file = File::open(input_path)
                .with_context(|| format!("Failed to open input file: {}", input_path))?;

            let mut buffer = [0; 50];
            input_file.read_exact(&mut buffer)?;
            input_file.rewind()?;
            let exif_marker: [u8; 6] = [0x45, 0x78, 0x69, 0x66, 0x00, 0x00];
            let exif_exists = buffer
                .windows(6)
                .position(|window| window == exif_marker)
                .is_some();
            let metadata = if exif_exists {
                Metadata::new_from_path(Path::new(input_path))?
            } else {
                Metadata::new()
            };

            let result =
                jpeg_compressor::compress(config.jpeg.as_ref(), &mut input_file, &metadata);
            match result {
                Ok(mut data) => {
                    if let Some(jpeg_config) = config.jpeg {
                        match jpeg_config.exif.as_str() {
                            "all" => {
                                // NOTE: Write "all" exif
                                metadata.write_to_vec(&mut data, FileExtension::JPEG)?;
                            }
                            "orientation" => {
                                // NOTE: Write "orientation" exif
                                let mut tag_iterator =
                                    metadata.get_tag(&ExifTag::Orientation(vec![]));
                                if let Some(exif_tag) = tag_iterator.next() {
                                    let mut new_metadata = Metadata::new();
                                    new_metadata.set_tag(exif_tag.clone());
                                    new_metadata.write_to_vec(&mut data, FileExtension::JPEG)?;
                                }
                            }
                            _ => {}
                        }
                    }

                    data
                }
                Err(e) => {
                    return Err(anyhow!(
                        "JPEG compression failed for file: {}. Error: {}",
                        input_path,
                        e
                    ));
                }
            }
        }
        _ => {
            return Err(anyhow!("Not supported image format"));
        }
    };

    let mut output_file = File::create(output_path)
        .with_context(|| format!("Failed to create output file: {}", output_path))?;
    output_file
        .write_all(&compressed_data)
        .with_context(|| format!("Failed to write to output file: {}", output_path))?;

    if verbose {
        println!("Output:");
        println!("\tFile name: {:?}", output_file_name);
        println!("Processing time: {:?}", now.elapsed());
        println!("End");
    }

    Ok(())
}
