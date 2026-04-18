use arthropod_mcp::ArthropodServer;

#[test]
fn test_havoc_mcp_context_poison() {
    let server = ArthropodServer::new();

    let server_clone = server.clone();
    let handle = std::thread::spawn(move || {
        let _guard = server_clone.context();
        panic!("Die!");
    });

    let _ = handle.join();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = server.context();
    }));

    assert!(result.is_ok(), "MCP server context lock is poisoned!");
}
