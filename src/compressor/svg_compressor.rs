use std::fs::File;

pub fn compress(input_file: &mut File) -> anyhow::Result<Vec<u8>> {
    // let options = usvg::Options {
    //     ..Default::default()       // デフォルト設定
    // };
    //
    // // SVGを解析してツリーを作成
    // let tree = usvg::Tree::from_data(&input, &options).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    //
    // // 最適化されたSVGを出力する
    // let mut output_file = File::create(output_path)?;
    // tree.write_to(&mut output_file, &options)?;

    Err(anyhow::anyhow!("Not implemented"))
}