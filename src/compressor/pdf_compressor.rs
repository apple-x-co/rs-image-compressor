use crate::config_json::PdfConfig;
use anyhow::anyhow;
use flate2::read::ZlibDecoder;
use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat};
use lopdf::{Dictionary, Document, Object, Stream};
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Write};

pub fn compress(input_file: &mut File, config: Option<&PdfConfig>) -> anyhow::Result<Vec<u8>> {
    let mut buf_reader = BufReader::new(input_file);
    let mut buffer = Vec::new();
    buf_reader.read_to_end(&mut buffer)?;

    let mut doc = Document::load_mem(&buffer)?;

    // NOTE: 未使用オブジェクトの削除
    doc.prune_objects();

    // NOTE: 不必要なストリームの削除
    doc.delete_zero_length_streams();

    // NOTE: ストリームをFlateで圧縮（非画像のテキスト系ストリームにも適用される）
    doc.compress();

    // NOTE: Infoを削除してプロパティを空にする
    if config.unwrap_or(&PdfConfig::default()).remove_info {
        doc.trailer.remove(b"Info");
    }

    // NOTE: Metadataを削除してプロパティを空にする
    if config.unwrap_or(&PdfConfig::default()).remove_metadata {
        doc.trailer.remove(b"Metadata");

        // NOTE: カタログ辞書（Root）から /Metadata を除去
        if let Ok(&Object::Reference(root_id)) = doc.trailer.get(b"Root") {
            if let Ok(Object::Dictionary(catalog)) = doc.get_object_mut(root_id) {
                catalog.remove(b"Metadata");
            }
        }
    }

    // NOTE: 未使用フォントの削除
    if config.unwrap_or(&PdfConfig::default()).remove_unuse_fonts {
        remove_unused_fonts(&mut doc)?;
    }

    // NOTE: 画像の圧縮
    compress_images(&mut doc, config)?;

    let out_buffer = Vec::new();
    let mut cursor = Cursor::new(out_buffer);
    doc.save_to(&mut cursor)?;

    Ok(cursor.into_inner().to_vec())
}

fn remove_unused_fonts(doc: &mut Document) -> anyhow::Result<()> {
    use std::collections::HashSet;
    use lopdf::ObjectId;

    let mut used_fonts = HashSet::new();
    
    // ページのリソースからフォント参照を収集
    for (_page_num, &page_id) in doc.get_pages().iter() {
        let (resources, _) = doc.get_page_resources(page_id)?;

        // Fontエントリを取得
        if let Some(resources) = resources {
            if let Ok(fonts_obj) = resources.get(b"Font") {
                match fonts_obj {
                    Object::Dictionary(fonts) => {
                        // 辞書から直接フォント参照を収集
                        for (_, font_obj) in fonts.iter() {
                            if let Object::Reference(ref_id) = font_obj {
                                used_fonts.insert(*ref_id);
                            }
                        }
                    }
                    Object::Reference(fonts_ref) => {
                        // 参照されている辞書からフォント参照を収集
                        if let Ok(Object::Dictionary(fonts)) = doc.get_object(*fonts_ref) {
                            for (_, font_obj) in fonts.iter() {
                                if let Object::Reference(ref_id) = font_obj {
                                    used_fonts.insert(*ref_id);
                                }
                            }
                        }
                    }
                    _ => {} // フォント辞書でないオブジェクトは無視
                }
            }
        }
    }

    // 未使用フォントを特定
    let font_ids: Vec<ObjectId> = doc.objects
        .iter()
        .filter_map(|(id, obj)| {
            if let Object::Dictionary(dict) = obj {
                // Type属性がFontで、使用中フォント集合に含まれていないものを抽出
                if let Ok(Object::Name(name)) = dict.get(b"Type") {
                    if name == b"Font" && !used_fonts.contains(id) {
                        return Some(*id);
                    }
                }
            }
            None
        })
        .collect();

    // 未使用フォントを削除
    for font_id in font_ids {
        doc.objects.remove(&font_id);
    }

    Ok(())
}

fn compress_images(doc: &mut Document, config: Option<&PdfConfig>) -> anyhow::Result<()> {
    let default_config = PdfConfig::default();
    let (png_min_quality, png_max_quality, jpeg_quality, jpeg_max_length) = match config {
        Some(config) => (
            config.png.quality_min,
            config.png.quality_max,
            config.jpeg.quality,
            config.jpeg.max_length,
        ),
        None => (
            default_config.png.quality_min,
            default_config.png.quality_max,
            default_config.jpeg.quality,
            default_config.jpeg.max_length,
        ),
    };

    let mut objects = Vec::new();

    for (object_id, object) in doc.objects.iter() {
        match object {
            Object::Stream(stream) => {
                let dict = &stream.dict;
                if let Ok(subtype) = dict.get(b"Subtype") {
                    if subtype == &Object::Name(b"Image".to_vec()) {
                        let filter = dict.get(b"Filter").and_then(Object::as_name).unwrap_or(b"");
                        let color_space = dict
                            .get(b"ColorSpace")
                            .and_then(Object::as_name)
                            .unwrap_or(b"");
                        let mut width = dict.get(b"Width").and_then(Object::as_i64).unwrap_or(0);
                        let mut height = dict.get(b"Height").and_then(Object::as_i64).unwrap_or(0);

                        if filter == b"DCTDecode" {
                            let decoded_img = image::load_from_memory(&stream.content)
                                .map_err(|e| anyhow!("Failed to decode JPEG image: {}", e))?
                                .to_rgb8();

                            let resized_img = if width > jpeg_max_length || height > jpeg_max_length
                            {
                                let ratio = jpeg_max_length as f32 / width.max(height) as f32;
                                width = (width as f32 * ratio).round() as i64;
                                height = (height as f32 * ratio).round() as i64;
                                image::imageops::resize(
                                    &decoded_img,
                                    width as u32,
                                    height as u32,
                                    FilterType::CatmullRom,
                                )
                            } else {
                                decoded_img
                            };

                            let rgb_data = resized_img.as_raw();

                            let color_space = mozjpeg::ColorSpace::JCS_RGB;
                            let mut compress = mozjpeg::Compress::new(color_space);
                            compress.set_quality(jpeg_quality as f32);
                            compress.set_size(width as usize, height as usize);
                            compress.set_scan_optimization_mode(
                                mozjpeg::ScanMode::AllComponentsTogether,
                            );
                            compress.set_optimize_coding(true);
                            compress.set_use_scans_in_trellis(false);
                            compress.set_smoothing_factor(0);

                            let mut started = compress
                                .start_compress(Vec::new())
                                .map_err(|e| anyhow!("Failed to start compress: {}", e))?;

                            let scanline_result = started.write_scanlines(rgb_data);
                            if scanline_result.is_err() {
                                let err = format!(
                                    "Failed to write scanline: {}",
                                    scanline_result.unwrap_err()
                                );
                                return Err(anyhow!(err));
                            }
                            let data = started
                                .finish()
                                .map_err(|e| anyhow!("Failed to finish compress: {}", e))?;

                            let new_stream = Stream::new(
                                Dictionary::from_iter(vec![
                                    (b"Type".to_vec(), Object::Name(b"XObject".to_vec())),
                                    (b"Subtype".to_vec(), Object::Name(b"Image".to_vec())),
                                    (b"Width".to_vec(), Object::Integer(width)),
                                    (b"Height".to_vec(), Object::Integer(height)),
                                    (b"Length".to_vec(), Object::Integer(data.len() as i64)),
                                    (b"ColorSpace".to_vec(), Object::Name(b"DeviceRGB".to_vec())),
                                    (b"BitsPerComponent".to_vec(), Object::Integer(8)),
                                    (b"Filter".to_vec(), Object::Name(b"DCTDecode".to_vec())),
                                    (b"Interpolate".to_vec(), Object::Boolean(true)),
                                ]),
                                data,
                            );

                            objects.push((*object_id, Object::Stream(new_stream)));
                        } else if filter == b"FlateDecode"
                            && (color_space == b"DeviceRGB" || color_space == b"DeviceGray")
                        {
                            let mut decoder = ZlibDecoder::new(&stream.content[..]);
                            let mut decoded_data = Vec::new();
                            decoder.read_to_end(&mut decoded_data)?;

                            let rgba_data = if color_space == b"DeviceRGB" {
                                decoded_data
                                    .chunks_exact(3)
                                    .map(|chunk| {
                                        imagequant::RGBA::new(chunk[0], chunk[1], chunk[2], 255)
                                    })
                                    .collect::<Vec<_>>()
                            } else {
                                // DeviceGray: gray → RGB
                                decoded_data
                                    .iter()
                                    .map(|g| imagequant::RGBA::new(*g, *g, *g, 255))
                                    .collect::<Vec<_>>()
                            };

                            let mut attr = imagequant::new();
                            attr.set_quality(png_min_quality, png_max_quality)?;

                            let mut liq_image = attr.new_image(
                                &rgba_data[..],
                                width as usize,
                                height as usize,
                                0.0,
                            )?;
                            let mut res = attr.quantize(&mut liq_image)?;
                            let (palette, pixels) = res.remapped(&mut liq_image)?;

                            let mut quantized_img =
                                image::ImageBuffer::new(width as u32, height as u32);
                            for (x, y, pixel) in quantized_img.enumerate_pixels_mut() {
                                let idx = (y * width as u32 + x) as usize;
                                let p = &palette[pixels[idx] as usize];
                                *pixel = image::Rgba([p.r, p.g, p.b, p.a]);
                            }

                            let dynamic_image = DynamicImage::ImageRgba8(quantized_img);

                            let mut data = Vec::new();
                            dynamic_image
                                .write_to(&mut Cursor::new(&mut data), ImageFormat::Png)?;

                            let mut encoder = flate2::write::ZlibEncoder::new(
                                Vec::new(),
                                flate2::Compression::default(),
                            );
                            encoder.write_all(&data)?;
                            let compressed_data = encoder.finish()?;

                            let new_stream = Stream::new(
                                Dictionary::from_iter(vec![
                                    (b"Type".to_vec(), Object::Name(b"XObject".to_vec())),
                                    (b"Subtype".to_vec(), Object::Name(b"Image".to_vec())),
                                    (b"Width".to_vec(), Object::Integer(width)),
                                    (b"Height".to_vec(), Object::Integer(height)),
                                    (
                                        b"Length".to_vec(),
                                        Object::Integer(compressed_data.len() as i64),
                                    ),
                                    (b"ColorSpace".to_vec(), Object::Name(b"DeviceRGB".to_vec())),
                                    (b"BitsPerComponent".to_vec(), Object::Integer(8)),
                                    (b"Filter".to_vec(), Object::Name(b"FlateDecode".to_vec())),
                                    (b"Interpolate".to_vec(), Object::Boolean(true)),
                                ]),
                                compressed_data,
                            );

                            objects.push((*object_id, Object::Stream(new_stream)));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    for (object_id, object) in objects {
        doc.objects.insert(object_id, object);
    }

    Ok(())
}