use std::net::TcpListener;
use std::net::TcpStream;
use std::io::Read;
use std::io::Write;

mod http_request;
mod http_response;
use crate::http_request::HTTPRequest;
use crate::http_response::HTTPResponse;

fn handle_client(client: &mut TcpStream) -> Result<(), ()> {
    let mut buffer: [u8; 4096] = [0; 4096];
    let bytes_read = client.read(&mut buffer).unwrap();

    if bytes_read == 0 {
        // Connection was closed
        return Err(())
    }

    println!("{}", bytes_read);
    let request = HTTPRequest::deserialise(&buffer[0..bytes_read]);
    let response = HTTPResponse::new_empty_body(200, "OK".to_string(), None);

    let _ = client.write(response.serialise().as_slice());

    Ok(())
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                loop {
                    if let Err(_e) = handle_client(&mut _stream) {
                        break;
                    }
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
