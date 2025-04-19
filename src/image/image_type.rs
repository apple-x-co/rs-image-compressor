use std::fs::File;
use std::io::{BufReader, Read};

#[derive(Debug, PartialEq, Clone)]
pub enum ImageType {
    GIF,
    PNG,
    JPEG,
    WEBP,
    HEIF,
}

pub fn image_type(buf_reader: &mut BufReader<File>) -> Option<ImageType> {
    let mut buffer = [0; 24];
    buf_reader.read(&mut buffer).unwrap();

    if infer::image::is_gif(&buffer) {
        return Some(ImageType::GIF);
    }

    if infer::image::is_png(&buffer) {
        return Some(ImageType::PNG);
    }

    if infer::image::is_jpeg(&buffer) {
        return Some(ImageType::JPEG);
    }

    if infer::image::is_webp(&buffer) {
        return Some(ImageType::WEBP);
    }

    if infer::image::is_heif(&buffer) {
        return Some(ImageType::HEIF);
    }

    None
}
