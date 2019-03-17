use std::{
    io,
    fs
};
use std::io::{
    Read,
    Seek,
    SeekFrom
};

use crate::error;


pub const GZIP_START_BYTES: [u8; 2] = [0x1f, 0x8b];


pub trait Compressed {
    fn is_gzipped(&mut self) -> error::Result<bool>;
}


impl Compressed for Vec<u8> {
    fn is_gzipped(&mut self) -> error::Result<bool> {
        Ok(self.starts_with(&GZIP_START_BYTES))
    }
}


impl Compressed for io::BufReader<fs::File> {
    fn is_gzipped(&mut self) -> error::Result<bool> {
        let mut start = [0; 2];
        self.read_exact(&mut start)?;
        self.seek(SeekFrom::Start(0))?;

        Ok(start.starts_with(&GZIP_START_BYTES))
    }
}
