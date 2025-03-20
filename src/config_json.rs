use serde::Deserialize;
use serde_json::Value;
use std::fs::read_to_string;

const JSON_SCHEMA_BYTES: &'static [u8] = include_bytes!("../schema/schema.json");

#[derive(Debug, Deserialize)]
pub struct PngConfig {
    pub quality: u8,
}

#[derive(Debug, Deserialize)]
pub struct JpegConfig {
    pub quality: u8,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub png: Option<PngConfig>,
    pub jpeg: Option<JpegConfig>,
}

impl Default for PngConfig {
    fn default() -> Self {
        Self { quality: 70 }
    }
}

impl Default for JpegConfig {
    fn default() -> Self {
        Self { quality: 70 }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            png: Some(PngConfig::default()),
            jpeg: Some(JpegConfig::default()),
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
