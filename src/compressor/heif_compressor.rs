use crate::config_json::HeifConfig;
use crate::error::CompressorError;
use anyhow::anyhow;
use libheif_rs::{
    ColorSpace, CompressionFormat, EncoderQuality, HeifContext, LibHeif, RgbChroma, StreamReader,
};
use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom};

pub fn compress(config: Option<&HeifConfig>, input_file: File) -> anyhow::Result<Vec<u8>> {
    let default_config = HeifConfig::default();
    let (quality, size) = match config {
        Some(config) => (config.quality, config.size.as_ref()),
        None => (default_config.quality, default_config.size.as_ref()),
    };

    let mut buf_reader = BufReader::new(input_file);
    let total_size = buf_reader.seek(SeekFrom::End(0))?;
    buf_reader.rewind()?;
    let stream_reader = StreamReader::new(buf_reader, total_size);

    let ctx = HeifContext::read_from_reader(Box::new(stream_reader))?;
    let handle = ctx.primary_image_handle()?;

    let lib_heif = LibHeif::new();

    let mut image = lib_heif.decode(&handle, ColorSpace::Rgb(RgbChroma::Rgba), None)?;

    if let Some(size) = size {
        image = image.scale(size.width, size.height, None)?;
    }

    let mut encoder = lib_heif.encoder_for_format(CompressionFormat::Hevc)?;
    
    if let Some(quality) = quality {
        encoder.set_quality(EncoderQuality::Lossy(quality))?;
    }

    let mut encode_context = HeifContext::new()?;
    encode_context.encode_image(&image, &mut encoder, None)?;

    let bytes = encode_context
        .write_to_bytes()
        .map_err(|e| anyhow!(CompressorError::HeifCompressError(e.to_string())))?;

    Ok(bytes)
}
