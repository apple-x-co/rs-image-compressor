use crate::error::CompressorError;
use anyhow::anyhow;
use std::fs::File;
use std::io::Read;
use usvg::{Indent, WriteOptions};

pub fn compress(input_path: &String) -> anyhow::Result<Vec<u8>> {
    let options = usvg::Options {
        ..Default::default()
    };

    let mut input_file = File::open(input_path)
        .map_err(|e| anyhow!(CompressorError::IoError(e)))?;

    let mut buffer = Vec::new();
    input_file
        .read_to_end(&mut buffer)
        .map_err(|e| anyhow!(CompressorError::IoError(e)))?;

    let tree = usvg::Tree::from_data(&buffer, &options)
        .map_err(|e| anyhow!(CompressorError::SvgCompressError(e.to_string())))?;

    let xml = tree.to_string(&WriteOptions{
        id_prefix: None,
        preserve_text: false,
        coordinates_precision: 0,
        transforms_precision: 0,
        use_single_quote: false,
        indent: Indent::None,
        attributes_indent: Indent::None,
    });

    Ok(xml.into_bytes())
}
