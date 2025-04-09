use crate::config_json::{GifConfig, SizeConfig};
use color_quant::NeuQuant;
use image::codecs::gif::{GifDecoder, GifEncoder, Repeat};
use image::imageops::FilterType;
use image::{AnimationDecoder, DynamicImage};
use image::{Frame as ImageFrame, GenericImageView};
use std::fs::File;
use std::io::{Cursor, Read};

pub fn compress(config: Option<&GifConfig>, input_file: &mut File) -> anyhow::Result<Vec<u8>> {
    let default_config = GifConfig::default();
    let (quality, size, max_colors, dithering, optimize_frames, loop_count) = match config {
        Some(config) => (
            config.quality,
            config.size.as_ref(),
            config.max_colors,
            config.dithering,
            config.optimize_frames,
            config.loop_count,
        ),
        None => (
            default_config.quality,
            default_config.size.as_ref(),
            default_config.max_colors,
            default_config.dithering,
            default_config.optimize_frames,
            default_config.loop_count,
        ),
    };

    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    let reader = Cursor::new(&buffer);
    let decoder = GifDecoder::new(reader)?;
    let frames = decoder.into_frames().collect_frames()?;
    let is_animated_gif = if frames.len() > 1 { true } else { false };

    if is_animated_gif {
        compress_animated_gif(
            &frames,
            quality,
            size,
            max_colors.unwrap_or(255),
            dithering.unwrap_or(false),
            optimize_frames,
            loop_count,
        )
    } else {
        compress_static_gif(
            &frames[0],
            quality,
            size,
            max_colors.unwrap_or(255),
            dithering.unwrap_or(false),
        )
    }
}

fn compress_static_gif(
    frame: &ImageFrame,
    quality: u8,
    size: Option<&crate::config_json::SizeConfig>,
    max_colors: u16,
    dithering: bool,
) -> anyhow::Result<Vec<u8>> {
    let mut dynamic_image = DynamicImage::from(frame.clone().into_buffer());

    // NOTE: リサイズの処理
    if let Some(size_config) = size {
        dynamic_image = match size_config.filter.as_str() {
            "nearest" => dynamic_image.resize(size_config.width, size_config.height, FilterType::Nearest),
            "triangle" => dynamic_image.resize(size_config.width, size_config.height, FilterType::Triangle),
            "catmull_rom" => dynamic_image.resize(size_config.width, size_config.height, FilterType::CatmullRom),
            "gaussian" => dynamic_image.resize(size_config.width, size_config.height, FilterType::Gaussian),
            "lanczos3" => dynamic_image.resize(size_config.width, size_config.height, FilterType::Lanczos3),
            _ => dynamic_image,
        }
    }

    // NOTE: 色数を減らす（量子化）
    dynamic_image = quantize_image(&dynamic_image, quality, max_colors, dithering)?;

    // NOTE: メモリ内でGIFを生成
    let mut output_buffer = Vec::new();
    {
        let mut encoder = GifEncoder::new(Cursor::new(&mut output_buffer));

        // NOTE: 新しいフレームを作成して書き込み
        let buffer = dynamic_image.to_rgba8();
        let new_frame = ImageFrame::from_parts(buffer, 0, 0, frame.delay());
        encoder.encode_frame(new_frame)?;
    }

    Ok(output_buffer)
}

fn compress_animated_gif(
    frames: &[ImageFrame],
    quality: u8,
    size: Option<&SizeConfig>,
    max_colors: u16,
    dithering: bool,
    optimize_frames: Option<bool>,
    loop_count: Option<u16>,
) -> anyhow::Result<Vec<u8>> {
    // NOTE: 出力バッファを作成
    let mut output_buffer = Vec::new();

    {
        let mut encoder = GifEncoder::new(Cursor::new(&mut output_buffer));

        // NOTE: ループ設定
        if let Some(loop_count) = loop_count {
            encoder.set_repeat(Repeat::Finite(loop_count))?;
        } else {
            encoder.set_repeat(Repeat::Infinite)?;
        }

        // NOTE: 各フレームを処理してエンコード
        for frame in frames {
            let mut dynamic_image = DynamicImage::from(frame.clone().into_buffer());

            if let Some(size) = size {
                dynamic_image = match size.filter.as_str() {
                    "nearest" => dynamic_image.resize(size.width, size.height, FilterType::Nearest),
                    "triangle" => { dynamic_image.resize(size.width, size.height, FilterType::Triangle) }
                    "catmull_rom" => { dynamic_image.resize(size.width, size.height, FilterType::CatmullRom) }
                    "gaussian" => { dynamic_image.resize(size.width, size.height, FilterType::Gaussian) }
                    "lanczos3" => { dynamic_image.resize(size.width, size.height, FilterType::Lanczos3) }
                    _ => dynamic_image,
                };
            }

            // NOTE: 色数を減らす（量子化）
            dynamic_image = quantize_image(&dynamic_image, quality, max_colors, dithering)?;

            // NOTE: 新しいフレームを作成して書き込み
            let buffer = dynamic_image.to_rgba8();
            let new_frame = ImageFrame::from_parts(buffer, 0, 0, frame.delay());
            encoder.encode_frame(new_frame)?;
        }
    }

    Ok(output_buffer)
}

fn quantize_image(
    img: &DynamicImage,
    quality: u8,
    max_colors: u16,
    dithering: bool,
) -> anyhow::Result<DynamicImage> {
    let rgba = img.to_rgba8();
    let (width, height) = img.dimensions();

    // NOTE: quality (0-100) から NeuQuant のファクター (1-30) に変換
    let nq_factor = match quality {
        0..=20 => 30,
        21..=40 => 20,
        41..=60 => 10,
        61..=80 => 5,
        _ => 1,
    };

    let mut quantizer = NeuQuant::new(nq_factor, max_colors as usize, rgba.as_raw().as_slice());

    let mut quantized_img = image::RgbaImage::new(width, height);

    // NOTE: ピクセルを量子化
    for (x, y, pixel) in rgba.enumerate_pixels() {
        let [r, g, b, a] = pixel.0;

        if a < 128 {
            // NOTE: 透明ピクセル
            quantized_img.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
        } else {
            // NOTE: 色を量子化 - 修正されたインターフェースに合わせる
            let pixel_data = [r, g, b, a];
            let idx = quantizer.index_of(&pixel_data) as usize;
            let palette = quantizer.color_map_rgba();
            let new_r = palette[idx * 4];
            let new_g = palette[idx * 4 + 1];
            let new_b = palette[idx * 4 + 2];
            quantized_img.put_pixel(x, y, image::Rgba([new_r, new_g, new_b, a]));
        }
    }

    // NOTE: ディザリングを実装する場合
    if dithering {
        // TODO:
    }

    Ok(DynamicImage::ImageRgba8(quantized_img))
}
