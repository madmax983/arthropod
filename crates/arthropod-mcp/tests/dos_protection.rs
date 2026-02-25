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

#[test]
fn test_large_legitimate_message() {
    // Elenchus: Verify Happy Path (Availability).
    // Ensure that a large message (e.g., 10MB) that is WITHIN the limit is accepted.
    // This prevents regression where the limit is set too low (e.g., 1KB).

    // 1. Start server
    let (_app, port) = start_tcp_server_with_port(0);

    // 2. Connect
    let mut stream =
        TcpStream::connect(format!("127.0.0.1:{}", port)).expect("Failed to connect to server");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("Failed to set timeout");

    // 3. Send 10MB message (well within 64MB limit)
    let size = 10 * 1024 * 1024;
    // Construct a valid JSON string to be nice, though garbage would also keep connection open (just parse error)
    // We send a partial string then fill with 'A's then close it.
    let mut huge_string = String::with_capacity(size + 100);
    huge_string.push_str("{\"jsonrpc\": \"2.0\", \"method\": \"ping\", \"params\": \"");

    // Fill with 'A's
    // We can't allocate 10MB 'A' string directly?
    // String::push_str handles it.
    // We'll do it in chunks to avoid allocating another 10MB buffer if possible, but huge_string already allocs.
    // Just append repeatedly.
    let chunk = "A".repeat(1024);
    for _ in 0..(size / 1024) {
        huge_string.push_str(&chunk);
    }
    huge_string.push_str("\"}\n");

    // Write it
    stream
        .write_all(huge_string.as_bytes())
        .expect("Failed to write legitimate message");

    // 4. Assert connection is still open
    // Attempt to read. If connection is open, we should get TimedOut (server waiting for next command).
    // If closed, we get EOF (Ok(0)).
    let mut buf = [0; 1];
    match stream.read(&mut buf) {
        Ok(0) => panic!("Connection closed by server! 10MB message was rejected (Limit too low?)."),
        Err(e)
            if e.kind() == std::io::ErrorKind::WouldBlock
                || e.kind() == std::io::ErrorKind::TimedOut =>
        {
            // Success! Connection is open.
            println!("Connection remains open for legitimate large message.");
        }
        Ok(_) => {
            // Data received? Also implies open.
        }
        Err(e) => panic!("Unexpected error: {}", e),
    }
}
