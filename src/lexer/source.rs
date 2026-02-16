use std::{
    fmt::Debug,
    fs::File,
    io::{self, Read},
    iter::Peekable,
    path::Path,
    str::Bytes,
};

pub trait ByteSource: Debug {
    type Error;
    fn peek(&mut self) -> Result<Option<u8>, &Self::Error>;
    fn next(&mut self) -> Result<Option<u8>, Self::Error>;
    fn report_error(e: &Self::Error) -> String;
}

pub struct FileSource {
    file_name: String,
    src: Peekable<io::Bytes<io::BufReader<File>>>,
}

impl FileSource {
    pub fn new<P>(path: P) -> Result<Self, io::Error>
    where
        P: AsRef<Path>,
    {
        let file_name = path
            .as_ref()
            .file_name()
            .expect("expected a file name")
            .to_str()
            .expect("expected a valid UTF-8 file name")
            .to_owned();

        let file = File::open(path)?;
        let reader = io::BufReader::new(file);
        Ok(Self { file_name, src: reader.bytes().peekable() })
    }
}

impl std::fmt::Debug for FileSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("FileSource: {}", self.file_name))
    }
}

impl ByteSource for FileSource {
    type Error = io::Error;

    fn peek(&mut self) -> Result<Option<u8>, &Self::Error> {
        match self.src.peek() {
            None => Ok(None),
            Some(Ok(b)) => Ok(Some(*b)),
            Some(Err(e)) => Err(e),
        }
    }

    fn next(&mut self) -> Result<Option<u8>, Self::Error> {
        self.src.next().transpose()
    }

    fn report_error(e: &Self::Error) -> String {
        e.to_string()
    }
}

pub struct StrSource<'a> {
    src: Peekable<Bytes<'a>>,
}

impl<'a> StrSource<'a> {
    pub fn new(src: &'a str) -> Self {
        Self { src: src.bytes().peekable() }
    }
}

impl std::fmt::Debug for StrSource<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("StrSource")
    }
}

impl ByteSource for StrSource<'_> {
    type Error = ();

    fn peek(&mut self) -> Result<Option<u8>, &Self::Error> {
        Ok(self.src.peek().copied())
    }

    fn next(&mut self) -> Result<Option<u8>, Self::Error> {
        Ok(self.src.next())
    }

    fn report_error(_e: &Self::Error) -> String {
        "".into()
    }
}
