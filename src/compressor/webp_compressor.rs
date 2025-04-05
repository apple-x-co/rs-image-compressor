use crate::config_json::WebpConfig;
use anyhow::{anyhow, Context};
use image::ImageReader;
use std::ffi::c_int;
use std::fs::File;
use std::io::BufReader;

pub fn compress(config: Option<&WebpConfig>, input_file: &mut File) -> anyhow::Result<Vec<u8>> {
    let default_config = WebpConfig::default();
    let (
        quality,
        method,
        target_size,
        target_psnr,
        lossless,
        alpha_compression,
        alpha_quality,
        pass,
        preprocessing,
        autofilter,
    ) = match config {
        Some(config) => (
            config.quality,
            config.method,
            config.target_size,
            config.target_psnr,
            config.lossless,
            config.alpha_compression,
            config.alpha_quality,
            config.pass,
            config.preprocessing,
            config.autofilter,
        ),
        None => (
            default_config.quality,
            default_config.method,
            default_config.target_size,
            default_config.target_psnr,
            default_config.lossless,
            default_config.alpha_compression,
            default_config.alpha_quality,
            default_config.pass,
            default_config.preprocessing,
            default_config.autofilter,
        ),
    };

    let reader = BufReader::new(input_file);
    let image_reader = ImageReader::new(reader)
        .with_guessed_format()
        .context("Failed to guess image format")?;

    let dynamic_image = image_reader.decode()?;

    let encoder = webp::Encoder::from_image(&dynamic_image)
        .map_err(|e| anyhow!("Failed to encode: {}", e))?;

    let mut webp_config = webp::WebPConfig::new().unwrap();
    webp_config.quality = quality as f32;

    if let Some(method) = method {
        webp_config.method = method as c_int;
    }

    if let Some(target_size) = target_size {
        webp_config.target_size = target_size as c_int;
    }

    if let Some(target_psnr) = target_psnr {
        webp_config.target_PSNR = target_psnr;
    }

    if let Some(lossless) = lossless {
        webp_config.lossless = if lossless { 1 } else { 0 };
    }

    if let Some(alpha_compression) = alpha_compression {
        webp_config.alpha_compression = if alpha_compression { 1 } else { 0 };
    }

    if let Some(alpha_quality) = alpha_quality {
        webp_config.alpha_quality = alpha_quality as c_int;
    }

    if let Some(pass) = pass {
        webp_config.pass = pass as c_int;
    }

    if let Some(preprocessing) = preprocessing {
        webp_config.preprocessing = preprocessing as c_int;
    }

    if let Some(autofilter) = autofilter {
        webp_config.autofilter = autofilter as c_int;
    }

    let webp_data = encoder.encode_advanced(&webp_config);
    match webp_data {
        Ok(webp_data) => Ok(webp_data.to_vec()),
        Err(e) => Err(anyhow!("Failed to encode: {:?}", e)),
    }
}
