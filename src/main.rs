mod matchmaker;
use async_std::{io, net::TcpListener, net::TcpStream, prelude::*, task};

use async_tls::TlsAcceptor;

use rustls::internal::pemfile;
use rustls::{NoClientAuth, ServerConfig};

use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::thread;

fn main() {
    println!("Creating server configuration...");
    // Loading the server certificate and private key.
    let configuration = load_config().expect("Failed to load server configuration.");
    println!("Server configuration created.");

    println!("Creating listening thread...");
    // Listening for incoming connections in another thread.
    thread::Builder::new()
        .name("ListeningThread".to_string())
        .spawn(move || task::block_on(start_listening(configuration)))
        .expect("Unable to create thread.");

    // Using the main thread to handle input from the user.
    task::block_on(async {
        handle_user_input().await;
    })
}

/// Loads server configuration in terms of certificate, key, settings. TODO: better error handling.
fn load_config() -> io::Result<ServerConfig> {
    // Read key.
    let key_path = Path::new("end.rsa");
    let mut buffer = BufReader::new(File::open(key_path).expect("Unable to open key.pem."));

    let mut private_key =
        pemfile::rsa_private_keys(&mut buffer).expect("Unable to read private key.");

    // Read certificates.
    let certificate_path = Path::new("end.cert");
    let mut buffer =
        BufReader::new(File::open(certificate_path).expect("Unable to open identity.pfx."));

    let certificates = pemfile::certs(&mut buffer).expect("Unable to open identity certificate.");

    // Create server configuration.
    let mut configuration = ServerConfig::new(NoClientAuth::new());

    configuration
        .set_single_cert(certificates, private_key.remove(0))
        .expect("Failed to set single certificate.");

    Ok(configuration)
}

async fn start_listening(configuration: ServerConfig) {
    println!("Listening thread created.");
    // Creating TCP listener.
    let tcp_listener = match TcpListener::bind("localhost:5568").await {
        Ok(tcp_listener) => tcp_listener,
        Err(error) => {
            println!("Server was unable to accept a connection. {:?}", error);
            return;
        }
    };

    // Creating TLS acceptor.
    let tls_acceptor = TlsAcceptor::from(Arc::new(configuration));

    println!("Listening for incoming connections...");
    loop {
        match tcp_listener.accept().await {
            Ok(connection) => {
                println!("TCP handshake with {:?} completed", connection.1);
                // Accept TLS connection and serve on a new thread.
                let tls_acceptor = tls_acceptor.clone();

                // Handle client in a new coroutine.
                task::spawn(handle_client(tls_acceptor, connection));
            }
            Err(error) => {
                println!("Unable to bind TCP listener. {:?}", error);
                return;
            }
        }
    }
}

/// Serves the client requests.
async fn handle_client(tls_acceptor: TlsAcceptor, connection: (TcpStream, SocketAddr)) {
    let mut tls_stream = match tls_acceptor.accept(connection.0).await {
        Ok(tls_stream) => tls_stream,
        Err(error) => {
            println!("Unable to establish a TLS connection. {:?}", error);
            return;
        }
    };

    println!("Accepted a TLS connection from {:?}.", connection.1);
    loop {
        // Read request. TODO: read request as flatbuffer.
        let mut buffer = [0; 20];
        tls_stream
            .read(&mut buffer)
            .await
            .expect("Unable to read from stream.");
        let request = String::from_utf8_lossy(&buffer[..]);
        let request = request.trim_end_matches(char::from(0));

        // TODO: define all possible requests.
        match request {
            "req1" => {
                // Answer to req1.
                println!("Received: {:?}", request);
                tls_stream
                    .write_all("ans1".as_bytes())
                    .await
                    .expect("Unable to write on stream.");
                tls_stream.flush().await.expect("Unable to flush stream.");
            }

            "enqueue" => {
                tls_stream
                    .write_all("Queued".as_bytes())
                    .await
                    .expect("Unable to write on stream.");
                tls_stream.flush().await.expect("Unable to flush stream.");
            }

            "close" => {
                // Terminate connection.
                println!("Received: {:?}", request);
                tls_stream
                    .write_all("Terminating".as_bytes())
                    .await
                    .expect("Unable to write on stream.");
                tls_stream.flush().await.expect("Unable to flush stream.");

                break;
            }

            _ => {
                // Default case. TODO: handle bad actors.
                println!("{:?} was not a valid request.", request);
                tls_stream
                    .write_all("Nope".as_bytes())
                    .await
                    .expect("Unable to write on stream.");
                tls_stream.flush().await.expect("Unable to flush stream.");
            }
        }
    }
}

/// This function will provide administration features to the server applications.
async fn handle_user_input() {
    // Initializations.
    let stdin = io::stdin();
    let mut buffer = String::new();

    // Main loop.
    loop {
        println!("Actively listening for user input.");
        buffer.clear();
        stdin
            .read_line(&mut buffer)
            .await
            .expect("Unable to read from stdin.");

        match buffer.trim() {
            "--exit" => {
                println!("Exiting");
                std::process::exit(0);
            }
            // TODO: add other functions.
            _ => println!("{:?} is not a valid command.", buffer),
        };
    }
}
