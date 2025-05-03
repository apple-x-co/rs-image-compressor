use serde::Deserialize;
use serde_json::Value;
use std::fs::read_to_string;

const JSON_SCHEMA_BYTES: &'static [u8] = include_bytes!("../schema/schema.json");

#[derive(Debug, Deserialize)]
pub struct SizeFilterConfig {
    pub width: u32,
    pub height: u32,
    pub filter: String,
}

#[derive(Debug, Deserialize)]
pub struct SizeConfig {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Deserialize)]
pub struct LibdeflaterConfig {
    pub compression: u8,
}

#[derive(Debug, Deserialize)]
pub struct ZopfliConfig {
    pub iterations: u8,
}

#[derive(Debug, Deserialize)]
pub struct LossyConfig {
    pub quality_min: u8,
    pub quality_max: u8,
    pub speed: Option<i32>,
    pub colors: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct PngConfig {
    pub quality: u8,
    pub size: Option<SizeFilterConfig>,
    pub strip: String,
    pub interlacing: String,
    pub optimize_alpha: bool,
    pub libdeflater: Option<LibdeflaterConfig>,
    pub zopfli: Option<ZopfliConfig>,
    pub lossy: Option<LossyConfig>,
}

#[derive(Debug, Deserialize)]
pub struct JpegConfig {
    pub quality: u8,
    pub size: Option<SizeFilterConfig>,
    pub scan_optimization_mode: Option<String>,
    pub progressive_mode: bool,
    pub optimize_coding: bool,
    pub use_scans_in_trellis: bool,
    pub smoothing_factor: u8,
    pub exif: String,
}

#[derive(Debug, Deserialize)]
pub struct WebpConfig {
    pub quality: u8,
    pub size: Option<SizeFilterConfig>,
    pub method: Option<u8>,
    pub target_size: Option<u8>,
    pub target_psnr: Option<f32>,
    pub lossless: Option<bool>,
    pub alpha_compression: Option<bool>,
    pub alpha_quality: Option<u8>,
    pub pass: Option<u8>,
    pub preprocessing: Option<u8>,
    pub autofilter: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct GifConfig {
    pub quality: u8,
    pub size: Option<SizeFilterConfig>,
    pub fast: Option<bool>,
    pub loop_count: Option<u16>,
    pub loop_speed: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct HeifConfig {
    pub quality: Option<u8>,
    pub size: Option<SizeConfig>,
}

#[derive(Debug, Deserialize)]
pub struct PdfPngConfig {
    pub quality_min: u8,
    pub quality_max: u8,
}

#[derive(Debug, Deserialize)]
pub struct PdfJpegConfig {
    pub quality: u8,
    pub max_length: i64,
}

#[derive(Debug, Deserialize)]
pub struct PdfConfig {
    pub remove_info: bool,
    pub remove_metadata: bool,
    pub remove_unuse_fonts: bool,
    pub png: PdfPngConfig,
    pub jpeg: PdfJpegConfig,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub png: Option<PngConfig>,
    pub jpeg: Option<JpegConfig>,
    pub webp: Option<WebpConfig>,
    pub gif: Option<GifConfig>,
    pub heif: Option<HeifConfig>,
    pub pdf: Option<PdfConfig>,
}

impl Default for PngConfig {
    fn default() -> Self {
        Self {
            quality: 2,
            size: None,
            strip: "all".into(),
            interlacing: "none".into(),
            optimize_alpha: false,
            libdeflater: None,
            zopfli: None,
            lossy: None,
        }
    }
}

impl Default for JpegConfig {
    fn default() -> Self {
        Self {
            quality: 70,
            size: None,
            scan_optimization_mode: Some("all_components_together".into()),
            progressive_mode: false,
            optimize_coding: true,
            use_scans_in_trellis: false,
            smoothing_factor: 0,
            exif: "none".into(),
        }
    }
}

impl Default for WebpConfig {
    fn default() -> Self {
        Self {
            quality: 70,
            size: None,
            method: None,
            target_size: None,
            target_psnr: None,
            lossless: Some(false),
            alpha_compression: Some(true),
            alpha_quality: None,
            pass: None,
            preprocessing: None,
            autofilter: None,
        }
    }
}

impl Default for GifConfig {
    fn default() -> Self {
        Self {
            quality: 75,
            size: None,
            fast: Some(false),
            loop_count: None,
            loop_speed: None,
        }
    }
}

impl Default for HeifConfig {
    fn default() -> Self {
        Self {
            quality: Some(50),
            size: None,
        }
    }
}

impl Default for PdfConfig {
    fn default() -> Self {
        Self {
            remove_info: true,
            remove_metadata: true,
            remove_unuse_fonts: false,
            png: PdfPngConfig {
                quality_min: 65,
                quality_max: 75,
            },
            jpeg: PdfJpegConfig {
                quality: 70,
                max_length: 1500,
            },
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            png: Some(PngConfig::default()),
            jpeg: Some(JpegConfig::default()),
            webp: Some(WebpConfig::default()),
            gif: Some(GifConfig::default()),
            heif: Some(HeifConfig::default()),
            pdf: Some(PdfConfig::default()),
        }
    }
}

pub fn parse(json_path: &str) -> Result<Config, &'static str> {
    let json_string = read_to_string(json_path).unwrap();
    let json: Value = serde_json::from_str(&json_string).unwrap();
    let schema = serde_json::from_slice(JSON_SCHEMA_BYTES).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();

    if !validator.validate(&json).is_ok() {
        return Err("Validation failed".into());
    }

    let config: Config = serde_json::from_str(&json_string).unwrap();

    Ok(config)
}
