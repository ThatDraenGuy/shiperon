use clap::{Parser, ValueEnum};
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
    /// Shiperon feature to enable
    #[arg(short, long)]
    feature: Vec<Feature>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Feature {
    /// Class cast expressions
    Cast,
    /// IO operations
    Io,
    /// String and Char support
    String,
    /// `super` keyword
    SuperKw,
    /// All features
    All,
}

fn main() {
    let args = Args::parse();

    let config = CompilerConfig {
        debug: args.debug,
        features: FeatureFlags {
            class_casting: args.feature.iter().any(|f| *f == Feature::Cast || *f == Feature::All),
            io: args.feature.iter().any(|f| *f == Feature::Io || *f == Feature::All),
            string: args.feature.iter().any(|f| *f == Feature::String || *f == Feature::All),
            super_kw: args.feature.iter().any(|f| *f == Feature::SuperKw || *f == Feature::All),
        },
    };

    println!("{config:?}");
}
