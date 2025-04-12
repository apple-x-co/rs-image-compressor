use crate::config_json::{GifConfig, SizeConfig};
use color_quant::NeuQuant;
use image::codecs::gif::{GifDecoder, GifEncoder, Repeat};
use image::imageops::FilterType;
use image::{AnimationDecoder, DynamicImage, Rgba, RgbaImage};
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

        // NOTE: フレーム最適化が有効かどうか
        let optimize_frames = optimize_frames.unwrap_or(false);

        // NOTE: 前のフレームを保持する変数（最適化用）
        let mut prev_processed_frame: Option<DynamicImage> = None;

        // NOTE: 各フレームを処理してエンコード
        for (i, frame) in frames.iter().enumerate() {
            // NOTE: 1. フレームの処理（リサイズと量子化）
            let mut dynamic_image = DynamicImage::from(frame.clone().into_buffer());

            // リサイズが必要な場合
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

            // NOTE: 量子化（色数を減らす）
            dynamic_image = quantize_image(&dynamic_image, quality, max_colors, dithering)?;

            // NOTE: 2. フレーム最適化（必要な場合）
            if optimize_frames && i > 0 && prev_processed_frame.is_some() {
                // 前のフレームと比較して差分を透明化
                dynamic_image = optimize_frame(&dynamic_image, prev_processed_frame.as_ref().unwrap())?;
            }

            // NOTE: 3. 新しいフレームを作成して書き込み
            let buffer = dynamic_image.to_rgba8();
            let new_frame = ImageFrame::from_parts(buffer, 0, 0, frame.delay());
            encoder.encode_frame(new_frame)?;

            // NOTE: 4. 現在のフレームを保存して次のイテレーションで使用
            prev_processed_frame = Some(dynamic_image);
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

    let quantizer = NeuQuant::new(nq_factor, max_colors as usize, rgba.as_raw().as_slice());

    let mut quantized_img = RgbaImage::new(width, height);

    // NOTE: ピクセルを量子化
    for (x, y, pixel) in rgba.enumerate_pixels() {
        let [r, g, b, a] = pixel.0;

        if a < 128 {
            // NOTE: 透明ピクセル
            quantized_img.put_pixel(x, y, Rgba([0, 0, 0, 0]));
        } else {
            // NOTE: 色を量子化 - 修正されたインターフェースに合わせる
            let pixel_data = [r, g, b, a];
            let idx = quantizer.index_of(&pixel_data) as usize;
            let palette = quantizer.color_map_rgba();
            let new_r = palette[idx * 4];
            let new_g = palette[idx * 4 + 1];
            let new_b = palette[idx * 4 + 2];
            quantized_img.put_pixel(x, y, Rgba([new_r, new_g, new_b, a]));
        }
    }

    // NOTE: ディザリングを実装する場合
    if dithering {
        apply_dithering(&mut quantized_img, &quantizer);
    }

    Ok(DynamicImage::ImageRgba8(quantized_img))
}

/// フレーム間の最適化を行う（差分エンコード）
fn optimize_frame(
    curr_frame: &DynamicImage,
    prev_frame: &DynamicImage,
) -> anyhow::Result<DynamicImage> {
    let (width, height) = curr_frame.dimensions();
    let curr_rgba = curr_frame.to_rgba8();
    let prev_rgba = prev_frame.to_rgba8();

    let mut optimized = RgbaImage::new(width, height);

    // 各ピクセルを比較し、変化のない部分を透明にする
    for (x, y, curr_pixel) in curr_rgba.enumerate_pixels() {
        let prev_pixel = prev_rgba.get_pixel(x, y);

        // ピクセルの比較（色と透明度の両方を考慮）
        if is_pixel_similar(curr_pixel, prev_pixel) {
            // 前のフレームと同じピクセルは透明に
            optimized.put_pixel(x, y, Rgba([0, 0, 0, 0]));
        } else {
            // 変化があるピクセルはそのまま
            optimized.put_pixel(x, y, *curr_pixel);
        }
    }

    // 最適化した結果、全て透明なら少なくとも1ピクセルは不透明にする
    // これは一部のGIFビューアが完全に透明なフレームを正しく処理できない場合があるため
    let mut all_transparent = true;
    for pixel in optimized.pixels() {
        if pixel.0[3] > 0 {
            all_transparent = false;
            break;
        }
    }

    if all_transparent {
        // 中央に1ピクセルだけ元の色を残す
        let center_x = width / 2;
        let center_y = height / 2;
        let original_pixel = curr_rgba.get_pixel(center_x, center_y);
        optimized.put_pixel(center_x, center_y, *original_pixel);
    }

    Ok(DynamicImage::ImageRgba8(optimized))
}

/// 2つのピクセルが視覚的に似ているかどうかを判定
fn is_pixel_similar(pixel1: &Rgba<u8>, pixel2: &Rgba<u8>) -> bool {
    let [r1, g1, b1, a1] = pixel1.0;
    let [r2, g2, b2, a2] = pixel2.0;

    // 透明度が低い場合は「似ている」と判断
    if a1 < 20 && a2 < 20 {
        return true;
    }

    // 両方のピクセルの透明度が異なる場合は「似ていない」
    if (a1 < 128 && a2 >= 128) || (a1 >= 128 && a2 < 128) {
        return false;
    }

    // 色の差を計算
    let r_diff = (r1 as i32 - r2 as i32).abs();
    let g_diff = (g1 as i32 - g2 as i32).abs();
    let b_diff = (b1 as i32 - b2 as i32).abs();

    // 色の差が閾値以下なら「似ている」と判断（許容誤差）
    r_diff <= 5 && g_diff <= 5 && b_diff <= 5
}

/// Floyd-Steinbergディザリングを適用
fn apply_dithering(img: &mut RgbaImage, quantizer: &NeuQuant) {
    let (width, height) = img.dimensions();
    if width < 2 || height < 2 {
        return; // 小さすぎる画像にはディザリングを適用しない
    }

    // オリジナル画像のコピーを作成
    let original = img.clone();

    for y in 0..height - 1 {
        for x in 1..width - 1 {
            let pixel = original.get_pixel(x, y);
            let [r, g, b, a] = pixel.0;

            if a < 128 {
                continue; // 透明ピクセルはスキップ
            }

            // 現在のピクセルを量子化
            let pixel_data = [r, g, b, a];
            let idx = quantizer.index_of(&pixel_data) as usize;
            let palette = quantizer.color_map_rgba();
            let new_r = palette[idx * 4];
            let new_g = palette[idx * 4 + 1];
            let new_b = palette[idx * 4 + 2];

            // 量子化誤差を計算
            let error_r = r as i32 - new_r as i32;
            let error_g = g as i32 - new_g as i32;
            let error_b = b as i32 - new_b as i32;

            // 隣接ピクセルに誤差を分散
            // Floyd-Steinbergディザリングパターン: 7/16, 3/16, 5/16, 1/16

            // 右のピクセル (x+1, y) に 7/16 の誤差を加える
            diffuse_error(img, x + 1, y, error_r, error_g, error_b, 7.0 / 16.0);

            // 左下のピクセル (x-1, y+1) に 3/16 の誤差を加える
            if x > 0 {
                diffuse_error(img, x - 1, y + 1, error_r, error_g, error_b, 3.0 / 16.0);
            }

            // 真下のピクセル (x, y+1) に 5/16 の誤差を加える
            diffuse_error(img, x, y + 1, error_r, error_g, error_b, 5.0 / 16.0);

            // 右下のピクセル (x+1, y+1) に 1/16 の誤差を加える
            diffuse_error(img, x + 1, y + 1, error_r, error_g, error_b, 1.0 / 16.0);
        }
    }
}

/// ディザリングのために隣接ピクセルに誤差を分散
fn diffuse_error(
    img: &mut RgbaImage,
    x: u32,
    y: u32,
    error_r: i32,
    error_g: i32,
    error_b: i32,
    factor: f32,
) {
    let (width, height) = img.dimensions();
    if x >= width || y >= height {
        return;
    }

    let pixel = img.get_pixel(x, y);
    let [r, g, b, a] = pixel.0;

    if a < 128 {
        return; // 透明ピクセルには誤差を分散しない
    }

    let new_r = clamp((r as i32 + (error_r as f32 * factor) as i32) as u8);
    let new_g = clamp((g as i32 + (error_g as f32 * factor) as i32) as u8);
    let new_b = clamp((b as i32 + (error_b as f32 * factor) as i32) as u8);

    img.put_pixel(x, y, Rgba([new_r, new_g, new_b, a]));
}

/// 値を0-255の範囲にクランプ
fn clamp(value: u8) -> u8 {
    value
}