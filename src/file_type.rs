use std::fs::File;
use std::io::{BufReader, Read};

#[derive(Debug, PartialEq, Clone)]
pub enum FileType {
    GIF,
    PNG,
    JPEG,
    WEBP,
    HEIF,
    PDF,
    XML,
}

pub fn detect(buf_reader: &mut BufReader<File>) -> Option<FileType> {
    let mut buffer = [0; 24];
    buf_reader.read(&mut buffer).unwrap();

    if infer::image::is_gif(&buffer) {
        return Some(FileType::GIF);
    }

    if infer::image::is_png(&buffer) {
        return Some(FileType::PNG);
    }

    if infer::image::is_jpeg(&buffer) {
        return Some(FileType::JPEG);
    }

    if infer::image::is_webp(&buffer) {
        return Some(FileType::WEBP);
    }

    if infer::image::is_heif(&buffer) {
        return Some(FileType::HEIF);
    }
    
    if infer::archive::is_pdf(&buffer) {
        return Some(FileType::PDF);
    }

    if infer::text::is_xml(&buffer) {
        return Some(FileType::XML);
    }

    None
}
