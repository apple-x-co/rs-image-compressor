use anyhow::Context;
use std::fs::File;
use std::io;
use std::io::Read;
use usvg::{Indent, WriteOptions};

pub fn compress(input_file: &mut File) -> anyhow::Result<Vec<u8>> {
    let options = usvg::Options {
        ..Default::default()
    };

    let mut buffer = Vec::new();
    input_file
        .read_to_end(&mut buffer)
        .context("Failed to read input file")?;

    let tree = usvg::Tree::from_data(&buffer, &options)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

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
