# rs-image-compressor

## Usage

```text
rs-image-compressor --input <INPUT> --output <OUTPUT>
```

```text
rs-image-compressor --input <INPUT> --output <OUTPUT> --config <CONFIG>
```

```text
rs-image-compressor --input <INPUT> --output <OUTPUT> --verbose
```

## Config

* [JSON Schema](https://raw.githubusercontent.com/apple-x-co/rs-image-compressor/refs/heads/main/schema/schema.json)

`png`

|                         | Type    | Range    | Enum                                         | Default | Note     |
|-------------------------|---------|----------|----------------------------------------------|---------|----------|
| quality                 | Integer | 1 .. 6   | -                                            | 2       | 1: High  |
| size.width              | Integer | -        | -                                            | -       |          |
| size.height             | Integer | -        | -                                            | -       |          |
| strip                   | String  | -        | none<br/>safe<br/>all                        | all     |          |
| interlacing             | String  | -        | none<br/>adam7                               | none    |          |
| optimize_alpha          | Bool    | -        | -                                            | false   |          |
| libdeflater.compression | Integer | 0 .. 12  | -                                            | -       | 12: High |
| zopfli.iterations       | Integer | 1 .. 15  | -                                            | -       |          |
| lossy.quality_min       | Integer | 0 .. 100 | -                                            | -       |          |
| lossy.quality_max       | Integer | 0 .. 100 | -                                            | -       |          |
| lossy.speed             | Integer | 1 .. 10  | -                                            | -       |          |
| lossy.colors            | Integer | -        | 4<br/>8<br/>16<br/>32<br/>64<br/>128<br/>256 | -       |          |

`jpeg`

|                        | Type    | Range    | Enum                                                    | Default                 | Note      |
|------------------------|---------|----------|---------------------------------------------------------|-------------------------|-----------|
| quality                | Integer | 1 .. 100 | -                                                       | 70                      | 100: High |
| size.width             | Integer | -        | -                                                       | -                       |           |
| size.height            | Integer | -        | -                                                       | -                       |           |
| scan_optimization_mode | String  | -        | all_components_together<br/>scan_per_component<br/>auto | all_components_together |           |
| progressive_mode       | Bool    | -        | -                                                       | false                   |           |
| optimize_coding        | Bool    | -        | -                                                       | true                    |           |
| use_scans_in_trellis   | Bool    | -        | -                                                       | false                   |           |
| smoothing_factor       | Integer | 0 .. 100 | -                                                       | 0                       |           |
| exif                   | String  | -        | none<br/>orientation<br/>all                            | none                    |           |


## Supported Images

* PNG
  * [oxipng](https://crates.io/crates/oxipng)
  * [pngquant](https://pngquant.org)
* JPEG
  * [mozjpeg](https://crates.io/crates/mozjpeg)