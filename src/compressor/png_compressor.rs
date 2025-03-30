use crate::config_json::PngConfig;
use anyhow::{anyhow, Context};
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageFormat, ImageReader};
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::num::NonZeroU8;

pub fn compress(config: Option<&PngConfig>, input_file: &mut File) -> anyhow::Result<Vec<u8>> {
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

        if let Some(colors) = lossy.colors {
            attr.set_max_colors(colors)?;
        }

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
