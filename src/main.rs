use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::time::Duration;
use std::thread;

mod http_request;
mod http_response;
mod http_compression;
use crate::http_request::HTTPRequest;
use crate::http_response::HTTPResponse;
use crate::http_compression::ContentEncoding;

fn router(request: &HTTPRequest) -> HTTPResponse {
    let response: HTTPResponse;
    let mut headers: HashMap<String, String> = HashMap::new();
    if let Some(accepted_encodings) = request.headers.get("Accept-Encoding") {
        let encoding = ContentEncoding::from_string(&accepted_encodings);
        println!("Encoding:{:?}", encoding);
        if encoding != ContentEncoding::IDENTITY {
            headers.insert("Content-Encoding".to_string(), encoding.to_string());
        }
    }

    if request.target == "/" {
        response = HTTPResponse::new_empty_body(200, "OK".to_string(), None);
    } else if request.target.starts_with("/echo/") {
        let content = request.target.strip_prefix("/echo/").unwrap();
        response = HTTPResponse { status: 200, reason: "OK".to_string(), headers: Some(headers), body: Some(content.to_string()) };
    } else if request.target == "/user-agent" {
        let content = request.headers.get("User-Agent").unwrap();
        response = HTTPResponse { status: 200, reason: "OK".to_string(), headers: Some(headers), body: Some(content.to_string()) };
    } else {
        response = HTTPResponse::new_empty_body(404, "Not Found".to_string(), None);
    }
    response
}

fn handle_client(client: &mut TcpStream) -> Result<(), ()> {
    let mut buffer: [u8; 4096] = [0; 4096];
    let read = client.read(&mut buffer);
    let bytes_read;
    match read {
        Ok(_bytes_read) => {
            bytes_read = _bytes_read;
        }
        Err(_e) => {
            // Close connection, timeout exceeded
            return Err(())
        }
    }

    if bytes_read == 0 {
        // Connection was closed
        return Err(())
    }

    println!("{}", bytes_read);
    let request = HTTPRequest::deserialise(&buffer[0..bytes_read]);
    let response = router(&request);
    let _ = client.write(response.serialise().as_slice());

    if let Some(value) = request.headers.get("Connection") {
        if value == "close" {
            // Client wants to close connection
            return Err(())
        }
    }

    Ok(())
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                println!("Accepted connection...");
                let timeout = Duration::new(60, 0);
                // Ignore the return since our duration is larger than 0
                let error_ = _stream.set_read_timeout(Some(timeout));
                thread::spawn(move || {
                    loop {
                        if let Err(_e) = handle_client(&mut _stream) {
                            break;
                        }
                    }
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
