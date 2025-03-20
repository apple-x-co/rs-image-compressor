# rs-image-compressor

## Usage

```text
rs-image-compressor --input <INPUT> --output <OUTPUT>
```

```text
rs-image-compressor --input <INPUT> --output <OUTPUT> --config <CONFIG>
```

## CONFIG

```json
{
  "$schema": "https://raw.githubusercontent.com/apple-x-co/rs-image-compressor/refs/heads/main/schema/schema.json",
  "png": {
    "quality": 3,
    "strip": "all",
    "interlacing": "none"
  },
  "jpeg": {
    "quality": 70
  }
}
```

## Supported Images

* PNG
* JPEG