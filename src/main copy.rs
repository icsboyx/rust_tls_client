use chrono::Local;
use openssl::ssl::{SslConnector, SslMethod, SslStream};
use std::{
    net::TcpStream,
    sync::{Arc, Mutex},
    thread,
    time::Duration, io::{Read, Write},
};

fn main() {
    let server = "localhost";
    let port = 6697;
    let address = format!("{}:{}", server, port);
    let tcp_stream = TcpStream::connect(address).unwrap();
    // tcp_stream.set_nonblocking(true).unwrap();
    tcp_stream.set_read_timeout(Some(Duration::new(0, 500))).unwrap();
    tcp_stream.set_write_timeout(Some(Duration::new(0, 500))).unwrap();

    let mut ssl_con = SslConnector::builder(SslMethod::tls()).unwrap();
    ssl_con
        .set_min_proto_version(Some(openssl::ssl::SslVersion::TLS1_3))
        .unwrap();
    ssl_con.set_verify(openssl::ssl::SslVerifyMode::NONE);
    let ssl_con = ssl_con.build();

    let ssl_stream: SslStream<TcpStream> = ssl_con.connect(server, tcp_stream).unwrap();
    let ssl_stream_shared = Arc::new(Mutex::new(ssl_stream));

    let ssl_stream_rx = Arc::clone(&ssl_stream_shared);
    let ssl_stream_tx = Arc::clone(&ssl_stream_shared);

    // Read from the SSL/TLS stream
    let mut buffer = [0; 1024]; // A buffer to hold the read data

    thread::spawn(move || 
        loop {
            loop {
            match ssl_stream_rx.lock().unwrap().read(&mut buffer) {
                Ok(bytes_read) => {
                if bytes_read > 0 {
                    let payload = String::from_utf8_lossy(&buffer);
                    println!("{}", payload);
                }

                }
                Err(ref err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(error) => {
                    println!("Error reading from the SSL/TLS stream: {:?}", error);
                    // Handle the error appropriately
                    // You might break the loop or take other actions
                    break;
                }
            }
        }
    });
    thread::spawn(move || loop {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let timestamp = format!("{}\n\r", timestamp);
        ssl_stream_tx
            .lock()
            .unwrap()
            .write_all(timestamp.as_bytes())
            .unwrap();
    });

    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
