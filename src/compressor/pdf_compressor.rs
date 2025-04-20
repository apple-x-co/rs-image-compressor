use lopdf::Document;
use std::fs::File;
use std::io::{BufReader, Cursor, Read};

pub fn compress(input_file: &mut File) -> anyhow::Result<Vec<u8>> {
    let mut buf_reader = BufReader::new(input_file);
    let mut buffer = Vec::new();
    buf_reader.read_to_end(&mut buffer)?;

    let mut doc = Document::load_mem(&buffer)?;
    doc.compress();

    let out_buffer = Vec::new();
    let mut cursor = Cursor::new(out_buffer);
    doc.save_to(&mut cursor)?;

    Ok(cursor.into_inner().to_vec())
}
