use crate::error::CompressorError;
use crate::io::file::read_file_bytes;
use anyhow::anyhow;
use usvg::{Indent, WriteOptions};

pub fn compress(input_path: &String) -> anyhow::Result<Vec<u8>> {
    let options = usvg::Options {
        ..Default::default()
    };

    let buffer = read_file_bytes(input_path)?;

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
