extern crate native_tls;

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
    // Open identity file and load its content into memory;
    let mut identity_file = File::open("identity.pfx").expect("Unable to open identity.pfx");

    let mut identity = vec![];
    identity_file.read_to_end(&mut identity).unwrap();
    let identity = Identity::from_pkcs12(&identity, "hunter2").unwrap();

    let tls_acceptor = TlsAcceptor::new(identity).unwrap();
    let tls_acceptor = Arc::new(tls_acceptor);

    let listener = TcpListener::bind("127.0.0.1:5568").expect("Couldn't bind the TcpListener.");
    println!("Listening for incoming connections...");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let tls_acceptor = tls_acceptor.clone();
                thread::spawn(move || {
                    let stream = tls_acceptor.accept(stream).unwrap();
                    handle_client(stream);
                });
            }
            Err(error) => {
                println!("Unable to accept a connection. {:?}", error);
            }
        };
    }
}

fn handle_client(mut stream: TlsStream<TcpStream>) {
    println!("Connection established.");
    loop {
        let mut buffer = [0; 512];
        stream.read(&mut buffer).unwrap();

        println!("Received: {:?}", String::from_utf8_lossy(&buffer[..]));
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
