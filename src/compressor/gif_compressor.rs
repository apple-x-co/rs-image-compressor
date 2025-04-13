use crate::config_json::GifConfig;
use anyhow::{anyhow, Context, Result};
use gifski::collector::ImgVec;
use gifski::{progress::NoProgress, Repeat, Settings};
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
    input_file.read_to_end(&mut buffer).context("Failed to read input file")?;

    // GIFファイルを解析
    let reader = Cursor::new(&buffer);
    let decoder = GifDecoder::new(reader).context("Failed to create GIF decoder")?;
    let frames = decoder.into_frames().collect_frames().context("Failed to collect frames")?;

    if frames.is_empty() {
        return Err(anyhow!("No frames found in GIF file"));
    }

    // gifski の設定
    let settings = Settings {
        quality: quality,
        repeat: match loop_count {
            Some(0) => Repeat::Finite(1),  // 0は表示を1回だけにする
            Some(count) => {
                // u16にキャストして、オーバーフローしないように確認
                let count_u16 = if count > u16::MAX as u16 {
                    u16::MAX
                } else {
                    count
                };
                Repeat::Finite(count_u16)
            },
            None => Repeat::Infinite,
        },
        fast: false, // 高品質優先
        width: None, // リサイズする場合は後で設定
        height: None,
    };

    // 結果を保存するバッファ
    let mut output_buffer = Vec::new();
    let writer = BufWriter::new(&mut output_buffer);

    // gifskiのコレクタを作成
    let (collector, writer_handle) = gifski::new(settings).context("Failed to initialize gifski")?;

    // 各フレームの処理と収集
    for (i, frame) in frames.iter().enumerate() {
        println!("i:{}", i);

        let mut dynamic_image = DynamicImage::from(frame.buffer().clone());

        // リサイズが必要な場合
        if let Some(size_config) = size {
            dynamic_image = match size_config.filter.as_str() {
                "nearest" => dynamic_image.resize(size_config.width, size_config.height, FilterType::Nearest),
                "triangle" => dynamic_image.resize(size_config.width, size_config.height, FilterType::Triangle),
                "catmull_rom" => dynamic_image.resize(size_config.width, size_config.height, FilterType::CatmullRom),
                "gaussian" => dynamic_image.resize(size_config.width, size_config.height, FilterType::Gaussian),
                "lanczos3" => dynamic_image.resize(size_config.width, size_config.height, FilterType::Lanczos3),
                _ => dynamic_image,
            };
        }

        // RGBA画像を取得
        let rgba_image = dynamic_image.to_rgba8();

        // フレームデータの準備
        let frame_data = prepare_frame_data(&rgba_image);

        // フレームの遅延時間（100分の1秒単位）
        let delay_denominator = 100_f64; // GIFの標準的な時間単位

        // フレームの遅延時間（ミリ秒単位）
        let delay_ms = frame.delay().numer_denom_ms().0 as f64 / 10.0;
        let frame_delay = delay_ms / delay_denominator;

        let img_vec = ImgVec::new(
            frame_data,
            rgba_image.width() as usize,
            rgba_image.height() as usize,
        );

        // フレームをコレクタに追加
        collector.add_frame_rgba(i, img_vec, frame_delay)
            .map_err(|e| anyhow!("Failed to add frame to gifski: {:?}", e))?;
    }

    // コレクタをドロップして処理を完了させる
    drop(collector);

    // 進捗報告なしで直接ライターハンドルを実行
    let mut no_progress = NoProgress {};
    writer_handle.write(writer, &mut no_progress)
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