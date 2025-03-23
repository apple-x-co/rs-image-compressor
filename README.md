# rs-image-compressor

## Usage

```text
rs-image-compressor --input <INPUT> --output <OUTPUT>
```

```text
rs-image-compressor --input <INPUT> --output <OUTPUT> --config <CONFIG>
```

## Config (JSON format)

```json
{
  "$schema": "https://raw.githubusercontent.com/apple-x-co/rs-image-compressor/refs/heads/main/schema/schema.json",
  "png": {
    "quality": 3,
    "strip": "all",
    "interlacing": "none",
    "optimize_alpha": true
  },
  "jpeg": {
    "quality": 70,
    "scan_optimization_mode": "none",
    "progressive_mode": false,
    "optimize_coding": true,
    "use_scans_in_trellis": false,
    "smoothing_factor": 0,
    "exif": "none"
  }
}
```

## Supported Images

* PNG
  * [oxipng](https://crates.io/crates/oxipng)
* JPEG
  * [mozjpeg](https://crates.io/crates/mozjpeg)