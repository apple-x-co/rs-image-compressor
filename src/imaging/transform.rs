use crate::config_json::SizeFilterConfig;
use image::DynamicImage;
use image::imageops::FilterType;

pub fn resize_image(image: &DynamicImage, config: &SizeFilterConfig) -> DynamicImage {
    match config.filter.as_str() {
        "nearest" => image.resize(config.width, config.height, FilterType::Nearest),
        "triangle" => image.resize(config.width, config.height, FilterType::Triangle),
        "catmull_rom" => image.resize(config.width, config.height, FilterType::CatmullRom),
        "gaussian" => image.resize(config.width, config.height, FilterType::Gaussian),
        "lanczos3" => image.resize(config.width, config.height, FilterType::Lanczos3),
        _ => image.clone(),
    }
}
