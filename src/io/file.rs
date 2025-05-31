use crate::error::{CompressorError, Result};
use crate::file_type::FileType;
use image::DynamicImage;
use image::ImageReader;
use std::fs::File;
use std::io::{BufReader, Read, Write};

pub fn read_image_from_file(file_path: &str) -> Result<DynamicImage> {
    let file = File::open(file_path)
        .map_err(|e| CompressorError::IoError(e))?;

    let reader = BufReader::new(file);
    let image_reader = ImageReader::new(reader)
        .with_guessed_format()
        .map_err(|e| CompressorError::ImageFormatError(e.to_string()))?;

    image_reader.decode()
        .map_err(|e| CompressorError::ImageDecodeError(e))
}

pub fn detect_file_type(file_path: &str) -> Result<FileType> {
    let file = File::open(file_path)
        .map_err(|e| CompressorError::IoError(e))?;

    let mut buf_reader = BufReader::new(file);
    let file_type = crate::file_type::detect(&mut buf_reader)
        .ok_or(CompressorError::UnknownFileFormat)?;

    Ok(file_type)
}

pub fn read_file_bytes(file_path: &str) -> Result<Vec<u8>> {
    let mut file = File::open(file_path)
        .map_err(|e| CompressorError::IoError(e))?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| CompressorError::IoError(e))?;

    Ok(buffer)
}

pub fn write_file_bytes(file_path: &str, data: &[u8]) -> Result<()> {
    let mut file = File::create(file_path)
        .map_err(|e| CompressorError::IoError(e))?;

    file.write_all(data)
        .map_err(|e| CompressorError::IoError(e))?;

    Ok(())
}