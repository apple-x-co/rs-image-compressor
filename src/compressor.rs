mod jpeg_compressor;
mod png_compressor;
mod webp_compressor;
mod gif_compressor;

use crate::config_json::Config;
use anyhow::{anyhow, Context, Result};
use image::ImageReader;
use image::ImageFormat;
use little_exif::exif_tag::ExifTag;
use little_exif::filetype::FileExtension;
use little_exif::metadata::Metadata;
use std::fs::File;
use std::io::{BufReader, Read, Seek, Write};
use std::path::Path;
use std::time::Instant;

pub fn compress(
    config: Config,
    verbose: bool,
    input_path: &String,
    output_path: &String,
) -> Result<()> {
    let now = Instant::now();
    let input_file_name = Path::new(input_path)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let output_file_name = Path::new(output_path)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();

    if verbose {
        println!("===== Start =====");
        println!("\n[Input]");
        println!("\tFile name: {}", input_file_name);
    }

    let input_file = File::open(input_path)
        .with_context(|| format!("Failed to open input file: {}", input_path))?;

    if verbose {
        let metadata = File::open(input_path)?.metadata()?;
        println!("\tSize: {} bytes", metadata.len());
    }

    let reader = BufReader::new(input_file);
    let image_reader = ImageReader::new(reader)
        .with_guessed_format()
        .context("Failed to guess image format")?;

    let image_format = match image_reader.format() {
        Some(format) => format,
        None => return Err(anyhow::anyhow!("Could not determine image format")),
    };

    if verbose {
        let (width, height) = image_reader.into_dimensions()?;
        println!("\tResolution: {}x{}", width, height);
        println!("\tMime type: {}", image_format.to_mime_type());
    }

    // NOTE: Compress image
    let compressed_data = match image_format {
        ImageFormat::Png => {
            if verbose {
                if let Some(png_config) = config.png.as_ref() {
                    println!("\n[Options]");
                    println!("\tQuality: {}", png_config.quality);

                    if let Some(size) = png_config.size.as_ref() {
                        println!("\tSize: {}x{}", size.width, size.width);
                    }

                    println!("\tStrip: {}", png_config.strip);
                    println!("\tInterlacing: {}", png_config.interlacing);
                    println!("\tOptimize alpha: {}", png_config.optimize_alpha);

                    if let Some(libdeflater) = png_config.libdeflater.as_ref() {
                        println!("\tLibdeflater:");
                        println!("\t\tCompression: {}", libdeflater.compression);
                    }

                    if let Some(zopfli) = png_config.zopfli.as_ref() {
                        println!("\tZopfli:");
                        println!("\t\tIterations: {}", zopfli.iterations);
                    }

                    if let Some(lossy) = png_config.lossy.as_ref() {
                        println!("\tLossy:");
                        println!("\t\tQuality_min: {}", lossy.quality_min);
                        println!("\t\tQuality_max: {}", lossy.quality_max);
                        if let Some(colors) = lossy.colors {
                            println!("\t\tColors: {}", colors);
                        }
                        if let Some(speed) = lossy.speed {
                            println!("\t\tSpeed: {}", speed);
                        }
                    }
                }
            }

            let mut input_file = File::open(input_path)
                .with_context(|| format!("Failed to open input file: {}", input_path))?;
            let result = png_compressor::compress(config.png.as_ref(), &mut input_file);
            match result {
                Ok(data) => data,
                Err(e) => {
                    return Err(anyhow!(
                        "PNG compression failed for file: {}. Error: {}",
                        input_path,
                        e
                    ));
                }
            }
        }
        ImageFormat::Jpeg => {
            if verbose {
                if let Some(jpeg_config) = config.jpeg.as_ref() {
                    println!("\n[Options]");
                    println!("\tQuality: {}", jpeg_config.quality);

                    if let Some(size) = jpeg_config.size.as_ref() {
                        println!("\tSize: {}x{}", size.width, size.width);
                    }

                    if let Some(scan_optimization_mode) = jpeg_config.scan_optimization_mode.as_ref() {
                        println!("\tScan optimization mode: {}", scan_optimization_mode);
                    }

                    println!("\tProgressive mode: {}", jpeg_config.progressive_mode);
                    println!("\tOptimize coding: {}", jpeg_config.optimize_coding);
                    println!("\tUse scans in trellis: {}", jpeg_config.use_scans_in_trellis);
                    println!("\tSmoothing factor: {}", jpeg_config.smoothing_factor);
                    println!("\tExif: {}", jpeg_config.exif);
                }
            }

            let mut input_file = File::open(input_path)
                .with_context(|| format!("Failed to open input file: {}", input_path))?;

            let mut buffer = [0; 50];
            input_file.read_exact(&mut buffer)?;
            input_file.rewind()?;
            let exif_marker: [u8; 6] = [0x45, 0x78, 0x69, 0x66, 0x00, 0x00];
            let exif_exists = buffer
                .windows(6)
                .position(|window| window == exif_marker)
                .is_some();
            let metadata = if exif_exists {
                Metadata::new_from_path(Path::new(input_path))?
            } else {
                Metadata::new()
            };

            let result =
                jpeg_compressor::compress(config.jpeg.as_ref(), &mut input_file, &metadata);
            match result {
                Ok(mut data) => {
                    if let Some(jpeg_config) = config.jpeg {
                        match jpeg_config.exif.as_str() {
                            "all" => {
                                // NOTE: Write "all" exif
                                metadata.write_to_vec(&mut data, FileExtension::JPEG)?;
                            }
                            "orientation" => {
                                // NOTE: Write "orientation" exif
                                let mut tag_iterator =
                                    metadata.get_tag(&ExifTag::Orientation(vec![]));
                                if let Some(exif_tag) = tag_iterator.next() {
                                    let mut new_metadata = Metadata::new();
                                    new_metadata.set_tag(exif_tag.clone());
                                    new_metadata.write_to_vec(&mut data, FileExtension::JPEG)?;
                                }
                            }
                            _ => {}
                        }
                    }

                    data
                }
                Err(e) => {
                    return Err(anyhow!(
                        "JPEG compression failed for file: {}. Error: {}",
                        input_path,
                        e
                    ));
                }
            }
        }
        ImageFormat::WebP => {
            if verbose {
                if let Some(webp_config) = config.webp.as_ref() {
                    println!("\n[Options]");
                    println!("\tQuality: {}", webp_config.quality);

                    // TODO:
                }
            }

            let mut input_file = File::open(input_path)
                .with_context(|| format!("Failed to open input file: {}", input_path))?;
            let result = webp_compressor::compress(config.webp.as_ref(), &mut input_file);
            match result {
                Ok(data) => data,
                Err(e) => {
                    return Err(anyhow!(
                        "WebP compression failed for file: {}. Error: {}",
                        input_path,
                        e
                    ));
                }
            }
        }
        ImageFormat::Gif => {
            if verbose {
                if let Some(gif_config) = config.gif.as_ref() {
                    println!("\n[Options]");
                    println!("\tQuality: {}", gif_config.quality);

                    if let Some(size) = gif_config.size.as_ref() {
                        println!("\tSize: {}x{}", size.width, size.height);
                    }

                    if let Some(fast) = gif_config.fast {
                        println!("\tFast: {}", fast);
                    }

                    if let Some(loop_count) = gif_config.loop_count {
                        println!("\tLoop count: {}", loop_count);
                    }
                }
            }

            let mut input_file = File::open(input_path)
                .with_context(|| format!("Failed to open input file: {}", input_path))?;
            let result = gif_compressor::compress(config.gif.as_ref(), &mut input_file);
            match result {
                Ok(data) => data,
                Err(e) => {
                    return Err(anyhow!(
                        "GIF compression failed for file: {}. Error: {}",
                        input_path,
                        e
                    ));
                }
            }
        }
        _ => {
            return Err(anyhow!("Not supported image format"));
        }
    };

    let mut output_file = File::create(output_path)
        .with_context(|| format!("Failed to create output file: {}", output_path))?;
    output_file
        .write_all(&compressed_data)
        .with_context(|| format!("Failed to write to output file: {}", output_path))?;

    if verbose {
        let metadata = File::open(input_path)?.metadata()?;
        let compressed_metadata = File::open(output_path)?.metadata()?;
        println!("\n[Result]");
        println!("\tBefore: {} bytes", metadata.len());
        println!("\tAfter: {} bytes", compressed_metadata.len());
        println!("\tRatio: {:.2} %", (compressed_metadata.len() as f64 / metadata.len() as f64) * 100.0);

        println!("\n[Output]");
        println!("\tFile name: {}", output_file_name);
        println!("\tProcessing time: {:?} sec", now.elapsed().as_secs_f64());
        println!("\n===== End =====");
    }

    Ok(())
}
