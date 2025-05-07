use std::collections::HashMap;
use crate::http_compression::ContentEncoding;
use crate::http_compression::compress;

pub struct HTTPResponse {
    pub status: u16,
    pub reason: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
}

impl HTTPResponse  {
    pub fn new_empty_body(status: u16, reason: String, headers: Option<HashMap<String, String>>) -> HTTPResponse {
        HTTPResponse { status: status, reason: reason, headers: headers, body: None}
    }

    pub fn serialise(self) -> Vec<u8> {
        let mut serialised = Vec::new();
        serialised.extend_from_slice(&format!("HTTP/1.1 {} {}\r\n", self.status, self.reason).as_bytes());

        if let Some(headers) = &self.headers {
            for (key, value) in headers {
                serialised.extend_from_slice(&format!("{}: {}\r\n", key, value).as_bytes());
                println!("Added header: {}:{}", key, value);
            }
        }

        if let Some(_body) = self.body {
            let body = _body.as_bytes();

            let compressed_result = self.headers
                .as_ref()
                .and_then(|headers| headers.get("Content-Encoding"))
                .map(|encoding_str| ContentEncoding::from_string(encoding_str))
                .and_then(|encoding| compress(encoding, body).ok());

            match compressed_result {
                Some(compressed) => {
                    serialised.extend_from_slice(format!("Content-Length: {}\r\n\r\n", compressed.len()).as_bytes());
                    serialised.extend(compressed);
                }
                None => {
                    serialised.extend_from_slice(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes());
                    serialised.extend_from_slice(body);
                }
            }
        } else {
            serialised.extend_from_slice(b"\r\n");
        }

        serialised
    }
}
