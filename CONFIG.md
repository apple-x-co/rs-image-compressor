# rs-image-compressor

## Config

👉 [JSON Schema](https://raw.githubusercontent.com/apple-x-co/rs-image-compressor/refs/heads/main/schema/schema.json)  
👉 [See samples](https://github.com/apple-x-co/rs-image-compressor-benchmark)

`PNG`

|                         | Type    | Range    | Enum                                         | Default | Note     |
|-------------------------|---------|----------|----------------------------------------------|---------|----------|
| quality                 | Integer | 1 .. 6   | -                                            | 2       | 1: High  |
| size.width              | Integer | -        | -                                            | -       |          |
| size.height             | Integer | -        | -                                            | -       |          |
| size.filter             | String  | -        | -                                            | -       |          |
| strip                   | String  | -        | none<br/>safe<br/>all                        | all     |          |
| interlacing             | String  | -        | none<br/>adam7                               | none    |          |
| optimize_alpha          | Bool    | -        | -                                            | false   |          |
| libdeflater.compression | Integer | 0 .. 12  | -                                            | -       | 12: High |
| zopfli.iterations       | Integer | 1 .. 15  | -                                            | -       |          |
| lossy.quality_min       | Integer | 0 .. 100 | -                                            | -       |          |
| lossy.quality_max       | Integer | 0 .. 100 | -                                            | -       |          |
| lossy.speed             | Integer | 1 .. 10  | -                                            | -       |          |
| lossy.colors            | Integer | -        | 4<br/>8<br/>16<br/>32<br/>64<br/>128<br/>256 | -       |          |

`JPEG`

|                        | Type    | Range    | Enum                                                    | Default                 | Note      |
|------------------------|---------|----------|---------------------------------------------------------|-------------------------|-----------|
| quality                | Integer | 1 .. 100 | -                                                       | 70                      | 100: High |
| size.width             | Integer | -        | -                                                       | -                       |           |
| size.height            | Integer | -        | -                                                       | -                       |           |
| size.filter            | String  | -        | -                                                       | -                       |           |
| scan_optimization_mode | String  | -        | all_components_together<br/>scan_per_component<br/>auto | all_components_together |           |
| progressive_mode       | Bool    | -        | -                                                       | false                   |           |
| optimize_coding        | Bool    | -        | -                                                       | true                    |           |
| use_scans_in_trellis   | Bool    | -        | -                                                       | false                   |           |
| smoothing_factor       | Integer | 0 .. 100 | -                                                       | 0                       |           |
| exif                   | String  | -        | none<br/>orientation<br/>all                            | none                    |           |

`WebP`

|                   | Type    | Range        | Enum | Default | Note                        |
|-------------------|---------|--------------|------|---------|-----------------------------|
| quality           | Integer | 1 .. 100     | -    | 75      | 100: High                   |
| size.width        | Integer | -            | -    | -       |                             |
| size.height       | Integer | -            | -    | -       |                             |
| size.filter       | String  | -            | -    | -       |                             |
| method            | Integer | 0 .. 6       | -    | -       | 6: High                     |
| target_size       | Integer |              | -    | -       |                             |
| target_psnr       | Integer | 25.0 .. 60.0 | -    | -       |                             |
| lossless          | Bool    |              | -    | -       |                             |
| alpha_compression | Bool    |              | -    | -       | false when lossless is true |
| alpha_quality     | Integer | 0 .. 100     | -    | -       | 100: High                   |
| pass              | Integer | 1 .. 100     | -    | -       |                             |
| preprocessing     | Integer | 0 .. 7       | -    | -       |                             |
| autofilter        | Bool    |              | -    | -       |                             |

`GIF`

|             | Type    | Range    | Enum | Default | Note      |
|-------------|---------|----------|------|---------|-----------|
| quality     | Integer | 1 .. 100 | -    | 75      | 100: High |
| size.width  | Integer | -        | -    | -       |           |
| size.height | Integer | -        | -    | -       |           |
| size.filter | String  | -        | -    | -       |           |
| fast        | Bool    | -        | -    | -       |           |
| loop_count  | Integer | -        | -    | -       |           |
| loop_speed  | Integer | -        | -    | -       |           |

`HEIF`

|                   | Type    | Range        | Enum | Default | Note                        |
|-------------------|---------|--------------|------|---------|-----------------------------|
| quality           | Integer | 1 .. 100     | -    | -       | 100: High                   |

`PDF`

|                    | Type    | Range    | Enum | Default | Note      |
|--------------------|---------|----------|------|---------|-----------|
| remove_info        | Bool    | -        | -    | true    | -         |
| remove_metadata    | Bool    | -        | -    | true    | -         |
| remove_unuse_fonts | Bool    | -        | -    | true    | -         |
| png.min_quality    | Integer | 1 .. 100 | -    | 65      | 100: High |
| png.max_quality    | Integer | 1 .. 100 | -    | 75      | 100: High |
| jpeg.quality       | Integer | 1 .. 100 | -    | 70      | 100: High |
| jpeg.max_length    | Integer | -        | -    | 1500    |           |