mod jpeg_compressor;
mod png_compressor;
mod webp_compressor;
mod gif_compressor;
mod heif_compressor;
mod pdf_compressor;
mod svg_compressor;

use crate::config_json::Config;
use crate::error::CompressorError;
use crate::file_type::FileType;
use crate::io::file::{detect_file_type, get_file_size, write_file_bytes};
use anyhow::{anyhow, Result};
use little_exif::exif_tag::ExifTag;
use little_exif::filetype::FileExtension;
use little_exif::metadata::Metadata;
use std::fs::File;
use std::io::{Read, Seek};
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

    let file_type = detect_file_type(input_path)?;

    if verbose {
        println!("===== Start =====");
        println!("\n[Input]");
        println!("\tFile name: {}", input_file_name);

        if let Ok(file_size) = get_file_size(input_path) {
            println!("\tSize: {} bytes", file_size);
        }
    }

    // NOTE: Compress a file
    let compressed_data = match file_type {
        FileType::PNG => {
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
                        println!("\t\tQuality min: {}", lossy.quality_min);
                        println!("\t\tQuality max: {}", lossy.quality_max);
                        if let Some(colors) = lossy.colors {
                            println!("\t\tColors: {}", colors);
                        }
                        if let Some(speed) = lossy.speed {
                            println!("\t\tSpeed: {}", speed);
                        }
                    }
                }
            }

            let result = png_compressor::compress(config.png.as_ref(), input_path);
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
        FileType::JPEG => {
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
                .map_err(|e| anyhow!(CompressorError::IoError(e)))?;

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
                jpeg_compressor::compress(config.jpeg.as_ref(), input_path, &metadata);
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
        FileType::WEBP => {
            if verbose {
                if let Some(webp_config) = config.webp.as_ref() {
                    println!("\n[Options]");
                    println!("\tQuality: {}", webp_config.quality);

                    // TODO:
                }
            }

            let result = webp_compressor::compress(config.webp.as_ref(), input_path);
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
        FileType::GIF => {
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

                    if let Some(loop_speed) = gif_config.loop_speed {
                        println!("\tLoop speed: {}", loop_speed);
                    }

                    if let Some(loop_count) = gif_config.loop_count {
                        println!("\tLoop count: {}", loop_count);
                    }
                }
            }

            let result = gif_compressor::compress(config.gif.as_ref(), input_path);
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
        FileType::HEIF => {
            if verbose {
                if let Some(heif_config) = config.heif.as_ref() {
                    println!("\n[Options]");

                    if let Some(quality) = heif_config.quality {
                        println!("\tQuality: {}", quality);
                    }
                }
            }

            let result = heif_compressor::compress(config.heif.as_ref(), input_path);
            match result {
                Ok(data) => data,
                Err(e) => {
                    return Err(anyhow!(
                        "HEIF/HEIC compression failed for file: {}. Error: {}",
                        input_path,
                        e
                    ));
                }
            }
        }
        FileType::PDF => {
            if verbose {
                if let Some(pdf_config) = config.pdf.as_ref() {
                    println!("\n[Options]");
                    println!("\tRemove info: {}", pdf_config.remove_info);
                    println!("\tRemove metadata: {}", pdf_config.remove_metadata);
                    println!("\tRemove unuse fonts: {}", pdf_config.remove_unuse_fonts);

                    println!("\tPng:");
                    println!("\t\tQuality Min: {}", pdf_config.png.quality_min);
                    println!("\t\tQuality Max: {}", pdf_config.png.quality_max);

                    println!("\tJpeg:");
                    println!("\t\tQuality: {}", pdf_config.jpeg.quality);
                    println!("\t\tMax length: {}", pdf_config.jpeg.max_length);
                }
            }

            let result = pdf_compressor::compress(input_path, config.pdf.as_ref());
            match result {
                Ok(data) => data,
                Err(e) => {
                    return Err(anyhow!(
                        "PDF compression failed for file: {}. Error: {}",
                        input_path,
                        e
                    ));
                }
            }
        }
        FileType::XML => {
            let result = svg_compressor::compress(input_path);
            match result {
                Ok(data) => data,
                Err(e) => {
                    return Err(anyhow!(
                        "SVG compression failed for file: {}. Error: {}",
                        input_path,
                        e
                    ));
                }
            }
        }
    };

    write_file_bytes(output_path, &compressed_data)?;

    if verbose {
        println!("\n[Result]");

        if let Ok(file_size) = get_file_size(input_path) {
            println!("\tBefore: {} bytes", file_size);
        }

        if let Ok(file_size) = get_file_size(input_path) {
            println!("\tAfter: {} bytes", file_size);
        }

        println!("\n[Output]");
        println!("\tFile name: {}", output_file_name);
        println!("\tProcessing time: {:?} sec", now.elapsed().as_secs_f64());
        println!("\n===== End =====");
    }

    Ok(())
}
