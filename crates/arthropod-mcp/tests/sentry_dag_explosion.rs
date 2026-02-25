use arthropod_mcp::context::McpFrameworkContext;
use arthropod_mcp::tools::Tool;
use arthropod_mcp::tools::scene::QueryHierarchyTool;
use render_engine::{NodeContent, SceneNode};
use serde_json::json;

#[test]
fn test_query_hierarchy_dag_explosion() {
    let mut ctx = McpFrameworkContext::new();
    let scene = ctx.scene_mut();
    let root = scene.root();

    // Create a dense tree: depth 7, branching factor 4
    // Total nodes = (4^8 - 1) / 3 = ~21,845 nodes.
    // This is > MAX_RESPONSE_NODES (5000).
    const DEPTH: usize = 7;
    const BRANCHING: usize = 4;

    fn build_tree(
        scene: &mut render_engine::Scene,
        parent: render_engine::NodeId,
        current_depth: usize,
    ) {
        if current_depth >= DEPTH {
            return;
        }
        for _ in 0..BRANCHING {
            let child = scene.add_node(parent, SceneNode::new(NodeContent::Empty));
            build_tree(scene, child, current_depth + 1);
        }
    }

    println!("Building dense tree...");
    build_tree(scene, root, 0);
    println!("Tree built. Nodes: {}", scene.nodes().count());

    let tool = QueryHierarchyTool;

    // Case 1: Request depth 100. Should fail because MAX_QUERY_DEPTH is 32.
    println!("Executing query with max_depth: 100...");
    let result = tool.execute(json!({ "max_depth": 100 }), &mut ctx);

    if let Err(e) = &result {
        println!("Got expected error: {}", e);
        assert!(e.to_string().contains("Max depth cannot exceed 32"));
    } else {
        panic!("Should have failed with max depth exceeded");
    }

    // Case 2: Request depth 30 (valid). Should return partial result because of MAX_RESPONSE_NODES (5000).
    println!("Executing query with max_depth: 30...");
    let result = tool.execute(json!({ "max_depth": 30 }), &mut ctx);

    if let Ok(val) = result {
        let json_str = serde_json::to_string(&val).unwrap();
        println!("Result size: {} bytes", json_str.len());
        // We expect the JSON to be relatively small because it was truncated at 5000 nodes.
        // 5000 nodes * ~100 bytes/node = ~500KB.
        // Before fix, it would be ~2MB (20k nodes).
        // Actually, since it returns None when limit reached, some branches are just cut off.

        // Let's verify that the structure is not empty but also not full.
        // It's hard to count nodes in JSON easily without parsing, but size check is a good proxy.
        assert!(
            json_str.len() < 2 * 1024 * 1024,
            "Response should be limited"
        );
    } else {
        println!("Query failed unexpectedly: {:?}", result.err());
        panic!("Query with valid depth should succeed (but be truncated)");
    }
}
