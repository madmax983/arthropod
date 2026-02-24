use arthropod_mcp::live::start_tcp_server_with_port;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

#[test]
fn test_unbounded_read_protection() {
    // 1. Start server on ephemeral port
    let (_app, port) = start_tcp_server_with_port(0);

    // 2. Connect to server
    let mut stream =
        TcpStream::connect(format!("127.0.0.1:{}", port)).expect("Failed to connect to server");

    // Set a timeout so the test doesn't hang if the server buffers indefinitely
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("Failed to set read timeout");

    // 3. Send a message larger than the 64MB limit
    // We send 64MB + 1KB of 'A's without a newline
    // This simulates a DoS attack (memory exhaustion)
    const LIMIT: usize = 64 * 1024 * 1024;
    const OVERFLOW: usize = 1024;
    let chunk_size = 1024 * 1024; // 1MB chunks
    let total_size = LIMIT + OVERFLOW;

    let chunk = vec![b'A'; chunk_size];

    // We write in chunks to avoid allocating 64MB buffer in test client
    let mut bytes_written = 0;
    while bytes_written < total_size {
        let remaining = total_size - bytes_written;
        let to_write = std::cmp::min(remaining, chunk_size);
        if stream.write_all(&chunk[..to_write]).is_err() {
            // If write fails, it means server closed connection already (Good!)
            break;
        }
        bytes_written += to_write;
    }

    // 4. Check if connection is closed
    // If the server is protected, it should close the connection when the limit is exceeded.
    // If vulnerable, it will keep buffering and waiting for newline, causing a timeout here.
    let mut buf = [0; 1];
    match stream.read(&mut buf) {
        Ok(0) => {
            println!("Connection closed by server (EOF) - Protected");
        }
        Err(e)
            if e.kind() == std::io::ErrorKind::ConnectionReset
                || e.kind() == std::io::ErrorKind::BrokenPipe
                || e.kind() == std::io::ErrorKind::ConnectionAborted =>
        {
            println!("Connection reset/broken by server - Protected");
        }
        Err(ref e)
            if e.kind() == std::io::ErrorKind::WouldBlock
                || e.kind() == std::io::ErrorKind::TimedOut =>
        {
            panic!("Server kept connection open (Vulnerable to unbounded read)");
        }
        Ok(_) => {
            panic!("Server sent unexpected data");
        }
        Err(e) => {
            println!("Other error: {:?} - Likely Protected", e);
        }
    }
}
