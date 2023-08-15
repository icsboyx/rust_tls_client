use chrono::Local;
use colored::*;
use ctrlc::set_handler;
use openssl::ssl::{SslConnector, SslMethod};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{process, thread};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    

    // Create an OpenSSL SSL connector.
    let mut connector = SslConnector::builder(SslMethod::tls()).unwrap();
    connector.set_verify(openssl::ssl::SslVerifyMode::NONE);
    let connector = connector.build();

    // Connect to localhost:667 using a TcpStream.
    let tcp_stream = TcpStream::connect("127.0.0.1:6697").unwrap();

    // Wrap the TcpStream in an SSL stream.
    let ssl_stream = connector.connect("localhost", tcp_stream).unwrap();

    // Get a Tcp stream out from the connector.
    let tcp_config_after_connect = ssl_stream.get_ref();
    // Set a nonblocking for read operations (optional), Must be done here not before connect.
    tcp_config_after_connect.set_nonblocking(true).unwrap();

    let ssl_stream = Arc::new(Mutex::new(ssl_stream));
    // Clone ssl_stream for tx and rx operations.
    let ssl_stream_rx = ssl_stream.clone();
    let ssl_stream_wr = ssl_stream.clone();

    // Perform read operations on the SSL stream
    let _ = thread::Builder::new().name("receiver_thread".to_string()).spawn(move || loop {
        let mut rx_payload = String::new();
        match ssl_stream_rx
            .lock()
            .unwrap()
            .read_to_string(&mut rx_payload)
        {
            Ok(n) if n == 0 => {
                println!("Server connection is closed. Exiting...");
                process::exit(1);
            }
            Ok(_) => {
                println!("Place holder for connection.");
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                } else {
                    eprintln!("Error: {}", e);
                }
            }
        }
        if !rx_payload.is_empty() {
            println!("{}", rx_payload);
        }
        thread::sleep(Duration::from_millis(100));
    });

    // Perform write operations on the SSL stream
    let _ = thread::Builder::new().name("sender_thread".to_string()).spawn(move || loop {
        let tx_payload = Local::now();
        let tx_payload = tx_payload.format("%Y-%m-%d %H:%M:%S\r\n").to_string();
        ssl_stream_wr
            .lock()
            .unwrap()
            .write_all(tx_payload.as_bytes())
            .unwrap();
        thread::sleep(Duration::from_secs(1));
    });
    
        // Register the CTRL+C signal handler
        set_handler(move || {
            println!(
                "{}",
                "\r\nCTRL+C signal received. Terminating..."
                    .red()
                    .bold()
                    .underline()
            );
            std::process::exit(0);
        })
        .unwrap();
    

    loop {
        thread::sleep(Duration::from_secs(10));
    }
}
