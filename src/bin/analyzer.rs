use std::{error::Error, fs::read_to_string};

use shiperon::{CompilerConfig, diagnostics::Renderable, process};

fn main() -> Result<(), Box<dyn Error>> {
    let input = read_to_string("samples/simple.po")?;
    let (registry, diagnostics, src) = process(&input, CompilerConfig::default());
    for item in &diagnostics {
        println!("{}\n", item.render(&src));
    }
    Ok(())
}
