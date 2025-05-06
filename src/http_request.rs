use std::collections::HashMap;

pub enum HTTPMethod {
    GET,
    POST
}

pub struct HTTPRequest {
    pub method: HTTPMethod,
    pub target: String,
    pub headers: HashMap<String, String>,
}

impl HTTPRequest {
    pub fn deserialise(buffer: &[u8]) -> HTTPRequest {
        let mut bytes_parsed: usize = 0;

        let mut request_line: String = String::new();
        for i in 1..buffer.len() {
            if buffer[i - 1] == b'\r' && buffer[i] == b'\n' {
                request_line = std::str::from_utf8(&buffer[0..i]).unwrap().to_string();
                bytes_parsed += i + 1;
                break;
            }
        }
        println!("Request: {}", request_line);
        let mut splits = request_line.split(" ");
        let method;
        match splits.next().unwrap() {
            "GET" => {
                method = HTTPMethod::GET
            }
            "POST" => {
                method = HTTPMethod::POST
            }
            _ => {
                method = HTTPMethod::GET
            }
        }
        let target: String = splits.next().unwrap().to_string();
        println!("Target: {}", target);

        let mut headers: HashMap<String, String> = HashMap::new();
        let mut start = bytes_parsed;
        for i in (bytes_parsed + 1)..buffer.len() {
            if buffer[i - 1] == b'\n' && buffer[i] == b'\r' {
                bytes_parsed = start + 3;
                break
            } else if buffer[i - 1] == b'\r' && buffer[i] == b'\n' {
                let header = std::str::from_utf8(&buffer[start..i]).unwrap().trim().to_string();
                let (key, value) = header.split_once(":").unwrap();
                headers.insert(key.trim().to_string(), value.trim().to_string());
                println!("Header: {}: {}", key, value);
                start = i;
            }
        }

        let body: String;
        if let Some(length) = headers.get("Content-Length") {
            let _length: usize = length.parse().unwrap();
            body = std::str::from_utf8(&buffer[bytes_parsed..(bytes_parsed + _length)]).unwrap().to_string();
            println!("Body: {}", body);
        }
         HTTPRequest { method: method, target: target, headers: headers }
    }
}
