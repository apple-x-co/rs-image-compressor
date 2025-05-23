use crate::error::CompressorError;
use crate::error::CompressorError::IoError;
use anyhow::anyhow;
use std::fs::File;
use std::io::Read;
use usvg::{Indent, WriteOptions};

pub fn compress(input_file: &mut File) -> anyhow::Result<Vec<u8>> {
    let options = usvg::Options {
        ..Default::default()
    };

    let mut buffer = Vec::new();
    input_file
        .read_to_end(&mut buffer)
        .map_err(|e| anyhow!(IoError(e)))?;

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
