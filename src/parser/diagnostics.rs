use crate::parser::ParserError;

#[derive(Default, Debug)]
pub struct Diagnostics {
    errors: Vec<ParserError>,
}

impl Diagnostics {
    pub fn add(&mut self, err: ParserError) {
        self.errors.push(err);
    }
}
