use arthropod_mcp::live::start_tcp_server_with_port;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::time::{Duration, sleep};

#[tokio::test]
async fn test_connection_flood_protection() {
    // 1. Start server on ephemeral port
    // This starts the server in a separate thread (std::thread)
    let (_app, port) = start_tcp_server_with_port(0);
    let addr = format!("127.0.0.1:{}", port);

    // 2. Spawn 100 clients (the limit we aim to enforce)
    let mut handles = vec![];
    let active_connections = Arc::new(AtomicUsize::new(0));

    // We want to verify that the server can handle 100 connections
    // AND that it REJECTS the 101st.
    // Currently (Red Phase), it will likely accept 101+, failing the test.

    println!("Spawning 105 connections...");
    for i in 0..105 {
        let addr = addr.clone();
        let active = active_connections.clone();
        handles.push(tokio::spawn(async move {
            match TcpStream::connect(&addr).await {
                Ok(mut stream) => {
                    active.fetch_add(1, Ordering::SeqCst);
                    // Hold the connection open
                    let mut buf = [0; 1024];
                    loop {
                        let read = stream.read(&mut buf).await;
                        match read {
                            Ok(0) => break, // EOF
                            Ok(_) => {}
                            Err(_) => break,
                        }
                    }
                    active.fetch_sub(1, Ordering::SeqCst);
                }
                Err(e) => {
                    println!("Connection {} failed: {}", i, e);
                }
            }
        }));
    }

    // Wait for connections to stabilize
    sleep(Duration::from_secs(2)).await;

    let count = active_connections.load(Ordering::SeqCst);
    println!("Active connections: {}", count);

    // 3. Assertions
    // If we have > 100 connections, the server is vulnerable (it didn't limit them).
    // We expect the limit to be 100.
    if count > 100 {
        panic!(
            "Server accepted {} connections! Vulnerable to DoS via thread exhaustion. Limit should be 100.",
            count
        );
    }

    // Elenchus: Verify availability (Happy Path).
    // If count is 0, the server might be down or rejecting everyone, which passes the above check
    // but fails the "functional server" requirement.
    if count < 90 {
        panic!(
            "Server only accepted {} connections! Availability issue or test flake. Expected ~100.",
            count
        );
    }

    // Cleanup
    for handle in handles {
        handle.abort();
    }
}
