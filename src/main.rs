mod compressor;
mod config_json;
mod image;

use crate::config_json::Config;
use anyhow::{Result, anyhow};
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
            Err(e) => return Err(anyhow!("{}", e)),
        }
    };

    compressor::compress(config, args.verbose, &args.input, &args.output)?;

    Ok(())
}
