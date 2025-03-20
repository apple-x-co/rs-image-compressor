use crate::config_json::{JpegConfig, PngConfig};
use anyhow::{Context, Result, anyhow};
use image::GenericImageView;
use image::codecs::jpeg::JpegEncoder;
use image::{ExtendedColorType, ImageReader};
use oxipng::{Interlacing, Options, PngError, StripChunks};
use std::fs::File;
use std::io::{BufReader, Read};

pub fn png_compressor(config: &Option<PngConfig>, input_file: &mut File) -> Result<Vec<u8>> {
    let mut reader = BufReader::new(input_file);
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;

    let default_config = PngConfig::default();
    let quality = match config {
        Some(config) => config.quality,
        None => default_config.quality,
    };
    let strip = match config {
        Some(config) => config.strip.as_str(),
        None => default_config.strip.as_str(),
    };
    let interlacing = match config {
        Some(config) => config.interlacing.as_str(),
        None => default_config.interlacing.as_str(),
    };

    let mut options = Options::from_preset(quality);
    options.strip = match strip {
        "safe" => StripChunks::Safe,
        "all" => StripChunks::All,
        _ => StripChunks::None,
    };
    options.interlace = match interlacing {
        "adam7" => Some(Interlacing::Adam7),
        _ => Some(Interlacing::None),
    };

    let png_result = oxipng::optimize_from_memory(&bytes, &options);
    match png_result {
        Ok(data) => Ok(data),
        Err(e) => match e {
            PngError::DeflatedDataTooLong(size) => Err(anyhow!("Deflated data too long: {}", size)),
            PngError::TimedOut => Err(anyhow!("PNG optimization timed out")),
            PngError::NotPNG => Err(anyhow!(
                "Invalid PNG header: Not a PNG file or file is corrupted"
            )),
            PngError::APNGNotSupported => Err(anyhow!("APNG format is not supported")),
            PngError::APNGOutOfOrder => Err(anyhow!("APNG chunks are out of order")),
            PngError::InvalidData => Err(anyhow!("Invalid PNG data")),
            PngError::TruncatedData => Err(anyhow!("Truncated PNG data")),
            PngError::ChunkMissing(chunk_type) => Err(anyhow!("Missing PNG chunk: {}", chunk_type)),
            PngError::InvalidDepthForType(bit_depth, color_type) => Err(anyhow!(
                "Invalid bit depth for color type: bit_depth={:?}, color_type={:?}",
                bit_depth,
                color_type
            )),
            PngError::IncorrectDataLength(expected, actual) => Err(anyhow!(
                "Incorrect data length: expected={}, actual={}",
                expected,
                actual
            )),
            PngError::C2PAMetadataPreventsChanges => Err(anyhow!("C2PA metadata prevents changes")),
            PngError::Other(message) => Err(anyhow!("PNG optimization failed: {}", message)),
            _ => Err(anyhow!("PNG optimization failed: {:?}", e)),
        },
    }
}

pub fn jpeg_compressor(config: &Option<JpegConfig>, input_file: &mut File) -> Result<Vec<u8>> {
    let reader = BufReader::new(input_file);
    let image_reader = ImageReader::new(reader)
        .with_guessed_format()
        .context("Failed to guess image format")?;
    let dynamic_image = image_reader.decode()?;

    let (width, height) = dynamic_image.dimensions();
    let rgb_image = dynamic_image.to_rgb8();

    let default_config = JpegConfig::default();
    let quality = match config {
        Some(config) => config.quality,
        None => default_config.quality,
    };

    let mut target: Vec<u8> = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut target, quality);
    encoder.encode(
        &rgb_image.into_raw(),
        width,
        height,
        ExtendedColorType::Rgb8,
    )?;

    Ok(target)
}
