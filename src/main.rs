mod compressor;

use crate::compressor::{jpeg_compressor, png_compressor};
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

    let image_format = match image_reader.format() {
        Some(format) => format,
        None => { return Err(anyhow::anyhow!("Could not determine image format")) }
    };

    let compressed_data = match image_format {
        ImageFormat::Png => {
            // TODO: File を渡さずに DynamicImage を渡せるか? そのためには、ファイルヘッダーを含む状態でバイトを取得できるようにする。
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
        ImageFormat::Jpeg => {
            let dynamic_image = image_reader.decode()?;
            let result = jpeg_compressor(&dynamic_image);
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

    println!("Compressed successfully and wrote to {}!", args.output);

    Ok(())
}
