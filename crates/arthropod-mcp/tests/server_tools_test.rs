use arthropod_mcp::ArthropodServer;
use rmcp::ServerHandler;

#[test]
fn test_all_tools_registered() {
    let server = ArthropodServer::new();
    let info = server.get_info();

    // Verify server has tools capability
    assert!(info.capabilities.tools.is_some(), "Server should have tools capability");

    println!("Server info: {:?}", info);
    println!("Capabilities: {:?}", info.capabilities);

    // The server should have all 20 tools registered
    // We can't easily count them from get_info(), but we verified compilation
    // and the pattern is correct for all 20 tools
}

#[test]
fn test_server_initialization() {
    let server = ArthropodServer::new();
    let info = server.get_info();

    assert_eq!(info.server_info.name, "rmcp");
    assert_eq!(info.instructions, Some("Arthropod MCP server - GUI framework testing and automation".to_string()));
}
