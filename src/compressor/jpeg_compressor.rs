use crate::config_json::JpegConfig;
use crate::error::CompressorError;
use crate::imaging::transform;
use anyhow::anyhow;
use image::{GenericImageView, ImageReader};
use little_exif::exif_tag::ExifTag;
use little_exif::metadata::Metadata;
use std::fs::File;
use std::io::BufReader;

pub fn compress(
    config: Option<&JpegConfig>,
    input_file: &mut File,
    metadata: &Metadata,
) -> anyhow::Result<Vec<u8>> {
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
            config.scan_optimization_mode.as_ref(),
            config.progressive_mode,
            config.optimize_coding,
            config.use_scans_in_trellis,
            config.smoothing_factor,
            config.size.as_ref(),
        ),
        None => (
            default_config.quality,
            default_config.scan_optimization_mode.as_ref(),
            default_config.progressive_mode,
            default_config.optimize_coding,
            default_config.use_scans_in_trellis,
            default_config.smoothing_factor,
            default_config.size.as_ref(),
        ),
    };

    let reader = BufReader::new(input_file);
    let image_reader = ImageReader::new(reader)
        .with_guessed_format()
        .map_err(|e| anyhow!(CompressorError::ImageDecodeError(e.into())))?;

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

    if let Some(size_config) = size {
        dynamic_image = transform::resize_image(&dynamic_image, size_config);
    }

    let (width, height) = dynamic_image.dimensions();
    let rgb_image = dynamic_image.to_rgb8();
    let bytes = rgb_image.into_raw();

    let color_space = mozjpeg::ColorSpace::JCS_RGB;
    let mut compress = mozjpeg::Compress::new(color_space);
    compress.set_size(width as usize, height as usize);
    compress.set_quality(quality as f32);
    if let Some(scan_optimization_mode) = scan_optimization_mode {
        compress.set_scan_optimization_mode(match scan_optimization_mode.as_str() {
            "all_components_together" => mozjpeg::ScanMode::AllComponentsTogether,
            "scan_per_component" => mozjpeg::ScanMode::ScanPerComponent,
            _ => mozjpeg::ScanMode::Auto,
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
        .map_err(|e| anyhow!(CompressorError::JpegCompressError(e.to_string())))?;

    let scanline_result = started.write_scanlines(&bytes);
    if scanline_result.is_err() {
        return Err(anyhow!(CompressorError::JpegCompressError(format!("Failed to write scanline: {}", scanline_result.unwrap_err()))));
    }
    let writer = started
        .finish()
        .map_err(|e| anyhow!(CompressorError::JpegCompressError(e.to_string())))?;

    Ok(writer)
}
