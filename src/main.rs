mod compressor;
mod config_json;

use crate::compressor::{jpeg_compressor, png_compressor};
use crate::config_json::Config;
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use image::{ImageFormat, ImageReader};
use std::fs::File;
use std::io::{BufReader, Write};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: String,

    #[arg(short, long)]
    config: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let config = if args.config.is_none() { Config::default() } else {
        match config_json::parse(args.config.unwrap().as_str()) {
            Ok(config) => config,
            Err(e) => return Err(anyhow!("{}", e)),
        }
    };

    let input_path = &args.input;
    let input_file = File::open(input_path)
        .with_context(|| format!("Failed to open input file: {}", input_path))?;

    let reader = BufReader::new(input_file);
    let image_reader = ImageReader::new(reader)
        .with_guessed_format()
        .context("Failed to guess image format")?;

    let image_format = match image_reader.format() {
        Some(format) => format,
        None => { return Err(anyhow::anyhow!("Could not determine image format")) }
    };

    let compressed_data = match image_format {
        ImageFormat::Png => {
            let mut input_file = File::open(input_path)
                .with_context(|| format!("Failed to open input file: {}", input_path))?;
            let result = png_compressor(config.png.as_ref(), &mut input_file);
            match result {
                Ok(data) => data,
                Err(e) => {
                    return Err(anyhow!("PNG compression failed for file: {}. Error: {}", input_path, e));
                }
            }
        },
        ImageFormat::Jpeg => {
            let mut input_file = File::open(input_path)
                .with_context(|| format!("Failed to open input file: {}", input_path))?;
            let result = jpeg_compressor(config.jpeg.as_ref(), &mut input_file);
            match result {
                Ok(data) => data,
                Err(e) => {
                    return Err(anyhow!("PNG compression failed for file: {}. Error: {}", input_path, e));
                }
            }
        }
        _ => {
            return Err(anyhow!("Not supported image format"));
        }
    };

    let mut output_file = File::create(&args.output)
        .with_context(|| format!("Failed to create output file: {}", args.output))?;
    output_file
        .write_all(&compressed_data)
        .with_context(|| format!("Failed to write to output file: {}", args.output))?;

    Ok(())
}
