use std::{error::Error, fs::read_to_string, path::Path};

use clap::Parser;
use shiperon::{CompilerConfig, config::FeatureFlags};

/// The one and only Shiperon Compiler
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Source file to compile
    src: String,
    /// Output file
    #[arg(short, long)]
    out: Option<String>,
    /// Debug compiler option
    #[arg(short, long)]
    debug: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let config = CompilerConfig {
        debug: args.debug,
        features: FeatureFlags { class_casting: true, io: false, string: true, super_kw: false },
        internal: false,
    };

    println!("{config:?}");

    let input = read_to_string(&args.src)?;
    shiperon::process(&input, Path::new(&args.out.unwrap()), config)?;

    Ok(())
}
