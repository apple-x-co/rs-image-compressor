use crate::config_json::{Config, JpegConfig, PngConfig};
use anyhow::{anyhow, Context, Result};
use image::imageops::FilterType;
use image::{DynamicImage, ImageReader};
use image::{GenericImageView, ImageFormat};
use little_exif::exif_tag::ExifTag;
use little_exif::filetype::FileExtension;
use little_exif::metadata::Metadata;
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek, Write};
use std::num::NonZeroU8;
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
    let default_config = PngConfig::default();
    let (quality, strip, interlacing, optimize_alpha, size, libdeflater, zopfli, lossy) =
        match config {
            Some(config) => (
                config.quality,
                config.strip.as_str(),
                config.interlacing.as_str(),
                config.optimize_alpha,
                config.size.as_ref(),
                config.libdeflater.as_ref(),
                config.zopfli.as_ref(),
                config.lossy.as_ref(),
            ),
            None => (
                default_config.quality,
                default_config.strip.as_str(),
                default_config.interlacing.as_str(),
                default_config.optimize_alpha,
                default_config.size.as_ref(),
                default_config.libdeflater.as_ref(),
                default_config.zopfli.as_ref(),
                default_config.lossy.as_ref(),
            ),
        };

    let reader = BufReader::new(input_file);
    let image_reader = ImageReader::new(reader)
        .with_guessed_format()
        .context("Failed to guess image format")?;

    let mut dynamic_image = image_reader.decode()?;

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

    if let Some(lossy) = lossy {
        let bitmap = dynamic_image
            .to_rgba8()
            .pixels()
            .map(|p| imagequant::RGBA::new(p.0[0], p.0[1], p.0[2], p.0[3]))
            .collect::<Vec<imagequant::RGBA>>();

        let mut attr = imagequant::new();
        attr.set_quality(lossy.quality_min, lossy.quality_max)?;

        if let Some(speed) = lossy.speed {
            attr.set_speed(speed)?;
        }

        let mut liq_image = attr.new_image(&bitmap[..], width as usize, height as usize, 0.0)?;
        let mut res = attr.quantize(&mut liq_image)?;
        let (palette, pixels) = res.remapped(&mut liq_image)?;

        let mut quantized_img = image::ImageBuffer::new(width, height);
        for (x, y, pixel) in quantized_img.enumerate_pixels_mut() {
            let idx = (y * width + x) as usize;
            let p = &palette[pixels[idx] as usize];
            *pixel = image::Rgba([p.r, p.g, p.b, p.a]);
        }

        dynamic_image = DynamicImage::ImageRgba8(quantized_img);
    }

    let mut bytes = Vec::new();
    dynamic_image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)?;

    let mut options = oxipng::Options::from_preset(quality);
    options.strip = match strip {
        "safe" => oxipng::StripChunks::Safe,
        "all" => oxipng::StripChunks::All,
        _ => oxipng::StripChunks::None,
    };
    options.interlace = match interlacing {
        "adam7" => Some(oxipng::Interlacing::Adam7),
        _ => Some(oxipng::Interlacing::None),
    };
    options.optimize_alpha = optimize_alpha;

    if let Some(libdeflater) = libdeflater {
        options.deflate = oxipng::Deflaters::Libdeflater {
            compression: libdeflater.compression,
        };
    } else if let Some(zopfli) = zopfli {
        options.deflate = oxipng::Deflaters::Zopfli {
            iterations: NonZeroU8::new(zopfli.iterations).unwrap(),
        };
    }

    let png_result = oxipng::optimize_from_memory(&bytes, &options);
    match png_result {
        Ok(data) => Ok(data),
        Err(e) => match e {
            oxipng::PngError::DeflatedDataTooLong(size) => {
                Err(anyhow!("Deflated data too long: {}", size))
            }
            oxipng::PngError::TimedOut => Err(anyhow!("PNG optimization timed out")),
            oxipng::PngError::NotPNG => Err(anyhow!(
                "Invalid PNG header: Not a PNG file or file is corrupted"
            )),
            oxipng::PngError::APNGNotSupported => Err(anyhow!("APNG format is not supported")),
            oxipng::PngError::APNGOutOfOrder => Err(anyhow!("APNG chunks are out of order")),
            oxipng::PngError::InvalidData => Err(anyhow!("Invalid PNG data")),
            oxipng::PngError::TruncatedData => Err(anyhow!("Truncated PNG data")),
            oxipng::PngError::ChunkMissing(chunk_type) => {
                Err(anyhow!("Missing PNG chunk: {}", chunk_type))
            }
            oxipng::PngError::InvalidDepthForType(bit_depth, color_type) => Err(anyhow!(
                "Invalid bit depth for color type: bit_depth={:?}, color_type={:?}",
                bit_depth,
                color_type
            )),
            oxipng::PngError::IncorrectDataLength(expected, actual) => Err(anyhow!(
                "Incorrect data length: expected={}, actual={}",
                expected,
                actual
            )),
            oxipng::PngError::C2PAMetadataPreventsChanges => {
                Err(anyhow!("C2PA metadata prevents changes"))
            }
            oxipng::PngError::Other(message) => {
                Err(anyhow!("PNG optimization failed: {}", message))
            }
            _ => Err(anyhow!("PNG optimization failed: {:?}", e)),
        },
    }
}

fn jpeg_compress(
    config: Option<&JpegConfig>,
    input_file: &mut File,
    metadata: &Metadata,
) -> Result<Vec<u8>> {
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
