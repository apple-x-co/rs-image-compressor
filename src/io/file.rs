use crate::error;
use image::DynamicImage;
use image::ImageReader;
use std::fs::File;
use std::io::BufReader;

pub fn read_image_from_file(file_path: &str) -> error::Result<DynamicImage> {
    let file = File::open(file_path)
        .map_err(|e| error::CompressorError::IoError(e))?;

    let reader = BufReader::new(file);
    let image_reader = ImageReader::new(reader)
        .with_guessed_format()
        .map_err(|e| error::CompressorError::ImageFormatError(e.to_string()))?;

    image_reader.decode()
        .map_err(|e| error::CompressorError::ImageDecodeError(e))
}