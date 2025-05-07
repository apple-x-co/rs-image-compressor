mod compressor;
mod config_json;
mod error;
mod file_type;

use crate::config_json::Config;
use crate::error::CompressorError;
use anyhow::{anyhow, Result};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: String,

    #[arg(short, long)]
    config: Option<String>,

    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let config = if args.config.is_none() {
        Config::default()
    } else {
        match config_json::parse(args.config.unwrap().as_str()) {
            Ok(config) => config,
            Err(e) => return Err(anyhow!(CompressorError::Other(e.to_string()))),
        }
    };

    compressor::compress(config, args.verbose, &args.input, &args.output)?;

    Ok(())
}
