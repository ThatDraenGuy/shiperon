use std::{error::Error, fs::read_to_string};

use shiperon::{CompilerConfig, process};

fn main() -> Result<(), Box<dyn Error>> {
    let input = read_to_string("samples/valid.po")?;
    process(&input, CompilerConfig::default());
    Ok(())
}
