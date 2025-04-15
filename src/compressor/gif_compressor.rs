use crate::config_json::GifConfig;
use anyhow::{Context, Result, anyhow};
use gifski::collector::ImgVec;
use gifski::{Repeat, Settings, progress::NoProgress};
use image::codecs::gif::GifDecoder;
use image::imageops::FilterType;
use image::{AnimationDecoder, DynamicImage, RgbaImage};
use rgb::RGBA8;
use std::fs::File;
use std::io::{BufWriter, Cursor, Read};

pub fn compress(config: Option<&GifConfig>, input_file: &mut File) -> Result<Vec<u8>> {
    // 設定値の取得
    let default_config = GifConfig::default();
    let (quality, size, _max_colors, _dithering, _optimize_frames, loop_count) = match config {
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

    // ファイルの内容を読み込み
    let mut buffer = Vec::new();
    input_file
        .read_to_end(&mut buffer)
        .context("Failed to read input file")?;

    // GIFファイルを解析
    let reader = Cursor::new(&buffer);
    let decoder = GifDecoder::new(reader).context("Failed to create GIF decoder")?;
    let frames = decoder
        .into_frames()
        .collect_frames()
        .context("Failed to collect frames")?;

    if frames.is_empty() {
        return Err(anyhow!("No frames found in GIF file"));
    }

    if frames.len() > 1000 {
        println!(
            "Warning: Large number of frames ({}). Processing may take time.",
            frames.len()
        );
    }

    // gifski の設定
    let settings = Settings {
        quality: quality,
        repeat: match loop_count {
            Some(0) => Repeat::Infinite,  // 0は無限ループを意味する（GIF仕様に準拠）
            Some(count) => {
                let count_u16 = (count as u16).min(u16::MAX);
                Repeat::Finite(count_u16)
            },
            None => Repeat::Infinite,
        },
        fast: false,
        width: match size {
            Some(size) => Some(size.width),
            None => None,
        },
        height: match size {
            Some(size) => Some(size.height),
            None => None,
        },
    };

    // 結果を保存するバッファ
    let mut output_buffer = Vec::new();
    let writer = BufWriter::new(&mut output_buffer);

    // gifskiのコレクタを作成
    let (collector, writer_handle) =
        gifski::new(settings).context("Failed to initialize gifski")?;

    // フレームの累積時間
    let mut current_presentation_timestamp = 0.0;

    // 各フレームの処理と収集
    for i in 0 .. frames.len() {
        let frame = &frames[i];

        if frame.buffer().is_empty() {
            println!("Skipping empty frame at index {}", i);
            continue;
        }

        {
            let mut dynamic_image = DynamicImage::from(frame.buffer().clone());

            // リサイズが必要な場合
            if let Some(size_config) = size {
                dynamic_image = match size_config.filter.as_str() {
                    "nearest" => {
                        dynamic_image.resize(size_config.width, size_config.height, FilterType::Nearest)
                    }
                    "triangle" => dynamic_image.resize(
                        size_config.width,
                        size_config.height,
                        FilterType::Triangle,
                    ),
                    "catmull_rom" => dynamic_image.resize(
                        size_config.width,
                        size_config.height,
                        FilterType::CatmullRom,
                    ),
                    "gaussian" => dynamic_image.resize(
                        size_config.width,
                        size_config.height,
                        FilterType::Gaussian,
                    ),
                    "lanczos3" => dynamic_image.resize(
                        size_config.width,
                        size_config.height,
                        FilterType::Lanczos3,
                    ),
                    _ => dynamic_image,
                };
            }

            // RGBA画像を取得
            let rgba_image = dynamic_image.to_rgba8();

            // フレームデータの準備
            let frame_data = prepare_frame_data(&rgba_image);

            // // GIFの遅延時間を正確に解釈する
            let delay_ms = frame.delay().numer_denom_ms().0 as f64;
            let frame_delay = (delay_ms / 1000.0).max(0.1);

            // ★★★ デバッグログを追加 ★★★
            // println!(
            //     "Adding frame {}/{}: width={}, height={}, delay_ms={}, frame_delay={}",
            //     i,
            //     frames.len() - 1,
            //     rgba_image.width(),
            //     rgba_image.height(),
            //     delay_ms,
            //     frame_delay
            // );

            let img_vec = ImgVec::new(
                frame_data,
                rgba_image.width() as usize,
                rgba_image.height() as usize,
            );

            // フレームをコレクタに追加
            if let Err(e) = collector.add_frame_rgba(i, img_vec, current_presentation_timestamp) {
                return Err(anyhow!("Failed to add frame {} (delay: {}s): {:?}",
                     i, frame_delay, e));
            }

            // 累積時間を更新
            current_presentation_timestamp += frame_delay;
        }
    }

    // コレクタをドロップして処理を完了させる
    drop(collector);

    // 進捗報告なしで直接ライターハンドルを実行
    let mut no_progress = NoProgress {};
    writer_handle
        .write(writer, &mut no_progress)
        .map_err(|e| anyhow!("Gifski writer error: {:?}", e))?;

    Ok(output_buffer)
}

/// RGBA画像を gifski に適した形式に変換
fn prepare_frame_data(rgba_image: &RgbaImage) -> Vec<RGBA8> {
    let width = rgba_image.width() as usize;
    let height = rgba_image.height() as usize;
    let mut frame_data = Vec::with_capacity(width * height);

    for pixel in rgba_image.pixels() {
        let [r, g, b, a] = pixel.0;
        frame_data.push(RGBA8::new(r, g, b, a));
    }

    frame_data
}
