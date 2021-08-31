use std::{
    str,
    fs::File,
    io::{
        self,
        BufRead,
        BufReader,
        Write,
    },
    collections::HashMap,
};

use libflate::gzip;

use http::{
    HttpClient,
    HttpClientError,
};

use crate::{
    EpgChannel,
    read_xml::{
        read_xml_tv,
        XmlReaderError,
    },
    write_xml::{
        write_xml_tv,
        XmlWriterError,
    },
};


// TODO: HashMap for codepage: language = codepage


#[derive(Debug, Error)]
pub enum EpgError {
    #[error_from("Epg IO: {}", 0)]
    Io(io::Error),
    #[error_from("Epg: {}", 0)]
    HttpClient(HttpClientError),
    #[error_from("Epg: {}", 0)]
    XmlReader(XmlReaderError),
    #[error_from("Epg: {}", 0)]
    XmlWriter(XmlWriterError),
    #[error_kind("Epg: unknown source type")]
    UnknownSourceType,
}


type Result<T> = std::result::Result<T, EpgError>;


fn is_gzip<R: io::BufRead>(src: &mut R) -> io::Result<bool> {
    static GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];
    let buf = src.fill_buf()?;
    Ok(buf.len() >= 2 && buf[.. 2] == GZIP_MAGIC)
}


#[derive(Default, Debug)]
pub struct Epg {
    pub channels: HashMap<String, EpgChannel>,
}


impl Epg {
    pub fn load<R: AsRef<str>>(&mut self, src: R) -> Result<()> {
        let src = src.as_ref();

        let mut i = src.splitn(2, "://");
        let scheme = i.next().unwrap();
        let (scheme, path) = match i.next() {
            Some(v) => (scheme, v),
            None => ("file", scheme),
        };

        match scheme {
            "file" => {
                let file = File::open(path)?;
                let mut buf = BufReader::new(file);
                self.read(&mut buf)
            }
            "http" | "https" => {
                let mut client = HttpClient::new(src)?;
                client.get()?;
                self.read(&mut client)
            }
            _ => Err(EpgError::UnknownSourceType),
        }
    }

    #[inline]
    pub fn read<R: BufRead>(&mut self, src: &mut R) -> Result<()> {
        if is_gzip(src)? {
            read_xml_tv(self, gzip::Decoder::new(src)?)?;
        } else {
            read_xml_tv(self, src)?;
        }

        Ok(())
    }

    #[inline]
    pub fn write<W: Write>(&self, dst: W) -> Result<()> {
        write_xml_tv(self, dst)?;
        Ok(())
    }
}
