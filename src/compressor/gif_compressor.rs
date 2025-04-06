use crate::config_json::GifConfig;
use std::fs::File;
use std::io::{BufReader, Cursor};
use anyhow::{anyhow, Context};
use image::{ImageFormat, ImageReader};

pub fn compress(
    config: Option<&GifConfig>,
    input_file: &mut File,
) -> anyhow::Result<Vec<u8>> {
    let reader = BufReader::new(input_file);
    let image_reader = ImageReader::new(reader)
        .with_guessed_format()
        .context("Failed to guess image format")?;

    let dynamic_image = image_reader.decode()?;

    // NOTE: temporary. Gifsicle? salzweg?
    let mut bytes = Vec::new();
    dynamic_image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Gif)?;
    // NOTE: temporary

    Ok(bytes)
}