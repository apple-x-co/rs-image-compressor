use crate::config_json::{Config, JpegConfig, PngConfig};
use anyhow::{anyhow, Context, Result};
use image::imageops::FilterType;
use image::ImageReader;
use image::{GenericImageView, ImageFormat};
use little_exif::exif_tag::ExifTag;
use little_exif::filetype::FileExtension;
use little_exif::metadata::Metadata;
use mozjpeg::{ColorSpace, Compress, ScanMode};
use oxipng::{Interlacing, Options, PngError, StripChunks};
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::Path;

pub fn compress(config: Config, input_path: &String, output_path: &String) -> Result<()> {
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
            let result = png_compress(config.png.as_ref(), &mut input_file);
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
            let metadata = Metadata::new_from_path(Path::new(input_path))?;

            let mut input_file = File::open(input_path)
                .with_context(|| format!("Failed to open input file: {}", input_path))?;
            let result = jpeg_compress(config.jpeg.as_ref(), &mut input_file, &metadata);
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

    Ok(())
}

fn png_compress(config: Option<&PngConfig>, input_file: &mut File) -> Result<Vec<u8>> {
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
    let optimize_alpha = match config {
        Some(config) => config.optimize_alpha,
        None => default_config.optimize_alpha,
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
    options.optimize_alpha = optimize_alpha;

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

fn jpeg_compress(
    config: Option<&JpegConfig>,
    input_file: &mut File,
    metadata: &Metadata,
) -> Result<Vec<u8>> {
    let reader = BufReader::new(input_file);
    let image_reader = ImageReader::new(reader)
        .with_guessed_format()
        .context("Failed to guess image format")?;

    let mut dynamic_image = image_reader.decode()?;

    if let Some(jpeg_config) = config {
        match jpeg_config.exif.as_str() {
            "none" => {
                let mut tag_iterator = metadata.get_tag(&ExifTag::Orientation(vec![]));
                if let Some(exif_tag) = tag_iterator.next() {
                    match exif_tag {
                        ExifTag::Orientation(values) => {
                            if let Some(value) = values.first() {
                                // NOTE: Rotation image by "orientation" exif
                                dynamic_image = match value {
                                    2 => dynamic_image.fliph(),
                                    3 => dynamic_image.rotate180(),
                                    4 => dynamic_image.flipv(),
                                    5 => dynamic_image.rotate90().fliph(),
                                    6 => dynamic_image.rotate90(),
                                    7 => dynamic_image.rotate270().fliph(),
                                    8 => dynamic_image.rotate270(),
                                    _ => dynamic_image,
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    let default_config = JpegConfig::default();
    let (
        quality,
        scan_optimization_mode,
        progressive_mode,
        optimize_coding,
        use_scans_in_trellis,
        smoothing_factor,
        size,
    ) = match config {
        Some(config) => (
            config.quality,
            config.scan_optimization_mode.as_str(),
            config.progressive_mode,
            config.optimize_coding,
            config.use_scans_in_trellis,
            config.smoothing_factor,
            config.size.as_ref(),
        ),
        None => (
            default_config.quality,
            default_config.scan_optimization_mode.as_str(),
            default_config.progressive_mode,
            default_config.optimize_coding,
            default_config.use_scans_in_trellis,
            default_config.smoothing_factor,
            default_config.size.as_ref(),
        ),
    };

    if let Some(size) = size {
        dynamic_image = match size.filter.as_str() {
            "nearest" => dynamic_image.resize(size.width, size.height, FilterType::Nearest),
            "triangle" => dynamic_image.resize(size.width, size.height, FilterType::Triangle),
            "catmull_rom" => dynamic_image.resize(size.width, size.height, FilterType::CatmullRom),
            "gaussian" => dynamic_image.resize(size.width, size.height, FilterType::Gaussian),
            "lanczos3" => dynamic_image.resize(size.width, size.height, FilterType::Lanczos3),
            _ => dynamic_image,
        }
    }

    let (width, height) = dynamic_image.dimensions();
    let rgb_image = dynamic_image.to_rgb8();
    let bytes = rgb_image.into_raw();

    let color_space = ColorSpace::JCS_RGB;
    let mut compress = Compress::new(color_space);
    compress.set_size(width as usize, height as usize);
    compress.set_quality(quality as f32);
    if scan_optimization_mode != "none" {
        compress.set_scan_optimization_mode(match scan_optimization_mode {
            "all_components_together" => ScanMode::AllComponentsTogether,
            "scan_per_component" => ScanMode::ScanPerComponent,
            _ => ScanMode::Auto,
        });
    }
    if progressive_mode {
        compress.set_progressive_mode();
    }
    compress.set_optimize_coding(optimize_coding);
    compress.set_use_scans_in_trellis(use_scans_in_trellis);
    compress.set_smoothing_factor(smoothing_factor);

    let mut started = compress
        .start_compress(Vec::new())
        .map_err(|e| anyhow!("Failed to start compress: {}", e))?;

    let scanline_result = started.write_scanlines(&bytes);
    if scanline_result.is_err() {
        let err = format!("Failed to write scanline: {}", scanline_result.unwrap_err());
        return Err(anyhow!(err));
    }
    let writer = started
        .finish()
        .map_err(|e| anyhow!("Failed to finish compress: {}", e))?;

    Ok(writer)
}

// NOTE: "JpegEncoder" は圧縮画像が元画像より大きくなる場合がある
// pub fn jpeg_compressor(config: &Option<JpegConfig>, input_file: &mut File) -> Result<Vec<u8>> {
//     let reader = BufReader::new(input_file);
//     let image_reader = ImageReader::new(reader)
//         .with_guessed_format()
//         .context("Failed to guess image format")?;
//     let dynamic_image = image_reader.decode()?;
//
//     let (width, height) = dynamic_image.dimensions();
//     let rgb_image = dynamic_image.to_rgb8();
//
//     let default_config = JpegConfig::default();
//     let quality = match config {
//         Some(config) => config.quality,
//         None => default_config.quality,
//     };
//
//     let mut target: Vec<u8> = Vec::new();
//     let mut encoder = JpegEncoder::new_with_quality(&mut target, quality);
//     encoder.encode(
//         &rgb_image.into_raw(),
//         width,
//         height,
//         ExtendedColorType::Rgb8,
//     )?;
//
//     Ok(target)
// }
