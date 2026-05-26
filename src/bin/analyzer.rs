use std::{error::Error, fs::read_to_string, path::Path};

use shiperon::{CompilerConfig, process};

fn main() -> Result<(), Box<dyn Error>> {
    let input = read_to_string("samples/small.po")?;
    process(&input, Path::new("out.ll"), CompilerConfig::default())?;
    Ok(())
}
