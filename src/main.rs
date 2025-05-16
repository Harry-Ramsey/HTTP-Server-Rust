use std::collections::HashMap;
use std::time::Duration;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::timeout;

use rustls::pki_types::pem::{Error, PemObject};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio_rustls::{rustls, TlsAcceptor};
use tokio_rustls::server::TlsStream;

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

async fn handle_client(client: &mut TlsStream<TcpStream>) -> Result<(), ()> {
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
            eprintln!("Client failed to respond within 60 seconds");
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

fn setup_tls() -> Result<TlsAcceptor, Error> {
    // Iterate through all certificates in a file
    // unwrap them and collect them in a vector.
    let certs = CertificateDer::pem_file_iter("./cert.pem")
        .unwrap()
        .map(|cert| cert.unwrap())
        .collect();

    let key = PrivateKeyDer::from_pem_file("./key.pem")?;

    // Create Server TLS configuration with no client authenticationn
    // and a single certificate.
    let config: rustls::ServerConfig = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key).unwrap();

    // Create acceptor which will do the serialise handshake
    // using the above server configuration.
    let acceptor = TlsAcceptor::from(Arc::new(config));
    Ok(acceptor)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").await?;
    println!("Listening on 127.0.0.1:4221");
    let acceptor;
    match setup_tls() {
        Ok(_acceptor) => {
            acceptor = _acceptor;
        }
        Err(_e) => {
            println!("Could not parse pem files");
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "TLS setup failed"));
        }
    }

    loop {
        let (mut stream, _) = listener.accept().await?;
        let mut tls_stream = acceptor.accept(stream).await?;
        println!("Accepted connection...");
        tokio::spawn(async move {
            loop {
                if let Err(_e) = handle_client(&mut tls_stream).await {
                    // Ignore return value since the connection
                    // has closed unexpectedly (client side) or
                    // is being closed (server side).
                    let _ = tls_stream.shutdown().await;
                    break;
                }
            }
        });
    }
}

