mod compressor;

use crate::compressor::png_compressor;
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

    #[arg(short, long, default_value = "")]
    config: String, // TODO: JSON Config
}

fn main() -> Result<()> {
    let args = Args::parse();

    let input_path = &args.input;
    let input_file = File::open(input_path)
        .with_context(|| format!("Failed to open input file: {}", input_path))?;

    let reader = BufReader::new(input_file);
    let image_reader = ImageReader::new(reader)
        .with_guessed_format()
        .context("Failed to guess image format")?;

    if let Some(image_format) = image_reader.format() {
        let compressed_data = match image_format {
            ImageFormat::Png => {
                let mut input_file = File::open(input_path)
                    .with_context(|| format!("Failed to open input file: {}", input_path))?;
                let result = png_compressor(&mut input_file);
                match result {
                    Ok(data) => data,
                    Err(e) => {
                        return Err(anyhow!("PNG compression failed for file: {}. Error: {}", input_path, e));
                    }
                }
            },
            _ => {
                return Err(anyhow!("Not supported image format"));
            }
        };

        let mut output_file = File::create(&args.output)
            .with_context(|| format!("Failed to create output file: {}", args.output))?;
        output_file
            .write_all(&compressed_data)
            .with_context(|| format!("Failed to write to output file: {}", args.output))?;

        println!("Compressed successfully and wrote to {}!", args.output);
    } else {
        return Err(anyhow::anyhow!("Could not determine image format"));
    }

    Ok(())
}
