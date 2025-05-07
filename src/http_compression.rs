use std::io::Read;
use flate2::Compression;
use flate2::bufread::GzEncoder;
use flate2::bufread::GzDecoder;
use flate2::bufread::DeflateEncoder;
use flate2::bufread::DeflateDecoder;

#[derive(PartialEq, Eq, Debug)]
pub enum ContentEncoding {
    GZIP,
    DEFLATE,
    IDENTITY,
}

impl ContentEncoding {
    pub fn from_string(encoding: &str) -> ContentEncoding {
        match encoding {
            "gzip" => ContentEncoding::GZIP,
            "deflate" => ContentEncoding::DEFLATE,
            &_ => ContentEncoding::IDENTITY,
        }
    }

    pub fn to_string(self) -> String {
        match self {
            ContentEncoding::GZIP => "gzip".to_string(),
            ContentEncoding::DEFLATE => "deflate".to_string(),
            ContentEncoding::IDENTITY => "identity".to_string(),
        }
    }
}

pub fn compress(encoding: ContentEncoding, buffer: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut encoded = Vec::new();
    match encoding {
        ContentEncoding::GZIP => {
            let mut encoder = GzEncoder::new(buffer, Compression::default());
            encoder.read_to_end(&mut encoded)?;
        }
        ContentEncoding::DEFLATE => {
            let mut encoder = DeflateEncoder::new(buffer, Compression::default());
            encoder.read_to_end(&mut encoded)?;
        }
        ContentEncoding::IDENTITY => {
            // Return since no encoding is required.
            encoded = buffer.to_vec();
        }
    }
    Ok(encoded)
}

pub fn decompress(encoding: ContentEncoding, buffer: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut decoded = Vec::new();
    match encoding {
        ContentEncoding::GZIP => {
            let mut decoder = GzDecoder::new(buffer);
            decoder.read_to_end(&mut decoded)?;
        }
        ContentEncoding::DEFLATE => {
            let mut decoder = DeflateDecoder::new(buffer);
            decoder.read_to_end(&mut decoded)?;
        }
        ContentEncoding::IDENTITY => {
            decoded = buffer.to_vec();
        }
    }
    Ok(decoded)
}

