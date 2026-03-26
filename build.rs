use std::{error::Error, fs, path::Path};

fn patch_bison_output(target: &Path) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(target)?;
    //top 10 free will usages
    let replaced = contents
        .replace(" /* 'fix quotes */", "")
        .replace(
            "impl<'src, S: ByteSource<'src>> Parser<'src, S: ByteSource<'src>>",
            "impl<'src, S: ByteSource<'src>> Parser<'src, S>",
        )
        .replace("type YYValue = Value", "type YYValue<'src> = Value<'src>")
        .replace("struct YYStackItem {", "struct YYStackItem<'src> {")
        .replace("value: YYValue,", "value: YYValue<'src>,")
        .replace("pub struct YYStack {", "pub struct YYStack<'src> {")
        .replace("stack: Vec<YYStackItem>,", "stack: Vec<YYStackItem<'src>>,")
        .replace("impl YYStack {", "impl<'src> YYStack<'src> {")
        .replace(
            "impl std::fmt::Display for YYStack {",
            "impl<'src> std::fmt::Display for YYStack<'src> {",
        )
        .replace("yystack: &mut YYStack", "yystack: &mut YYStack<'src>")
        .replace(
            //literally how the fuck did I do this. this is insane. who tf came up with lifetimes. iyky
            "fn owned_value_at(&mut self, i: usize) -> YYValue",
            "fn owned_value_at<'a>(&'a mut self, i: usize) -> YYValue<'src>",
        );
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
