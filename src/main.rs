use std::collections::HashMap;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::timeout;

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

async fn handle_client(client: &mut TcpStream) -> Result<(), ()> {
    let mut buffer: [u8; 4096] = [0; 4096];

    let bytes_read;
    match timeout(Duration::from_secs(60), client.read(&mut buffer)).await {
        Ok(Ok(n)) => {
            bytes_read = n;
        },
        Ok(Err(e)) => {
            eprintln!("Failed to read from socket error: {}", e);
            return Err(());
        }
        Err(_) => {
            println!("Client failed to respond within 60 seconds");
            return Err(());
        },
    }


    if bytes_read == 0 {
        // Connection was closed
        return Err(())
    }

    println!("{}", bytes_read);
    let request = HTTPRequest::deserialise(&buffer[0..bytes_read]);
    let response = router(&request);
    let _ = client.write(response.serialise().as_slice()).await;

    if let Some(value) = request.headers.get("Connection") {
        if value == "close" {
            // Client wants to close connection
            return Err(())
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").await?;

    loop {
        let (mut stream, _) = listener.accept().await?;
        println!("Accepted connection...");
        tokio::spawn(async move {
            loop {
                if let Err(_e) = handle_client(&mut stream).await {
                    break;
                }
            }
        });
    }
}

