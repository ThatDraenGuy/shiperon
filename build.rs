use std::{error::Error, fs, path::Path};

fn patch_bison_output(target: &Path) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(target)?;
    let replaced = contents
        .replace("impl<S: ByteSource> Parser<S: ByteSource>", "impl<S: ByteSource> Parser<S>");
    fs::write(target, replaced)?;
    Ok(())
}

fn main() {
    const PARSE_Y: &str = "src/parser/parse.y";
    const PARSE_RS: &str = "src/parser/parse.rs";

    println!("cargo:rerun-if-changed={PARSE_Y}");
    println!("Generating parse.rs");

    match rust_bison_skeleton::process_bison_file(Path::new(PARSE_Y)) {
        Ok(_) => patch_bison_output(Path::new(PARSE_RS)).unwrap(),
        // Ok(_) => {},
        Err(err) => {
            eprintln!("Failed to generate grammar.\n{err:#?}");
            std::process::exit(1);
        },
    }
}
