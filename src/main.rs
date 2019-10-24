extern crate native_tls;

mod matchmaker;
use native_tls::{Identity, TlsAcceptor, TlsStream};
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

fn main() {
    match thread::Builder::new()
        .name("ListeningThread".to_string())
        .spawn(move || start_listening())
    {
        Ok(_) => {}
        Err(error) => println!("Unable to create thread.{:?}", error),
    };

    handle_user_input();
}

fn start_listening() {
    // Open identity file and load its content into memory.
    let mut identity_file = File::open("identity.pfx").expect("Unable to open identity.pfx");

    let mut server_identity = vec![];
    identity_file.read_to_end(&mut server_identity).unwrap();
    let server_identity = Identity::from_pkcs12(&server_identity, "krahos").unwrap();

    // Creating TLS listener.
    let tls_acceptor = TlsAcceptor::new(server_identity).unwrap();
    let tls_acceptor = Arc::new(tls_acceptor);

    // Creating TCP listener.
    let listener = TcpListener::bind("127.0.0.1:5568").expect("Couldn't bind the TcpListener.");
    println!("Listening for incoming connections...");
    let mut i = 0;
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Accept TLS connection and serve on a new thread. TODO: use a thread pool.
                let tls_acceptor = tls_acceptor.clone();
                match thread::Builder::new().name(i.to_string()).spawn(move || {
                    let stream = tls_acceptor.accept(stream).unwrap();
                    handle_client(stream);
                }) {
                    Ok(_) => {}
                    Err(error) => {
                        println!("Unable to create thread to serve new client. {:?}", error)
                    }
                }
            }
            Err(error) => {
                println!("Unable to accept a connection. {:?}", error);
            }
        };
        i += 1;
    }
}

/// Serves the client requests.
fn handle_client(mut stream: TlsStream<TcpStream>) {
    println!("Connection established.");
    loop {
        // Read request. TODO: read request as flatbuffer.
        let mut buffer = [0; 20];
        stream.read(&mut buffer).unwrap();
        let request = String::from_utf8_lossy(&buffer[..]);
        let request = request.trim_end_matches(char::from(0));

        // TODO: define all possible requests.
        match request {
            "req1" => {
                // Answer to req1.
                println!("Received: {:?}", request);
                stream.write_all("ans1".as_bytes()).unwrap();
                stream.flush().unwrap();
            }

            "enqueue" => {
                stream.write_all("Queued".as_bytes()).unwrap();
                stream.flush().unwrap();
            }

            "close" => {
                // Terminate connection.
                println!("Received: {:?}", request);
                stream.write_all("Terminating".as_bytes()).unwrap();
                stream.flush().unwrap();

                break;
            }

            _ => {
                // Default case. TODO: handle bad actors.
                println!("{:?} was not a valid request.", request);
                stream.write_all("Nope".as_bytes()).unwrap();
                stream.flush().unwrap();
            }
        }
    }
}

/// This function will provide administration features to the server applications.
fn handle_user_input() {
    // Initializations.
    let stdin = io::stdin();
    let mut buffer = String::new();

    // Main loop.
    loop {
        println!("Awaiting input:");
        buffer.clear();
        stdin.read_line(&mut buffer).unwrap();

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
