//! MCP-enhanced framework context.
//!
//! Wraps FrameworkContext with additional capabilities for MCP operations:
//! - Signal registry for remote control
//! - Named node lookup
//! - Performance tracking
//! - Visual testing integration

use arthropod_ecs::FrameworkContext;
use hashbrown::HashMap;
use render_engine::{NodeId, Scene, backend::RectInstance};
use std::time::{Duration, Instant};

use crate::registry::SignalRegistry;

/// Enhanced framework context with MCP capabilities
pub struct McpFrameworkContext {
    /// Inner framework context (ECS + Scene)
    inner: FrameworkContext,

    /// Signal registry for remote control
    signal_registry: SignalRegistry,

    /// Named node lookup (name -> NodeId)
    named_nodes: HashMap<String, NodeId>,

    /// Performance measurement history
    perf_history: Vec<PerfMeasurement>,
}

/// Performance measurement record
#[derive(Debug, Clone)]
pub struct PerfMeasurement {
    /// Measurement type
    pub kind: PerfKind,

    /// Duration of operation
    pub duration: Duration,

    /// Timestamp
    pub timestamp: Instant,
}

/// Type of performance measurement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerfKind {
    /// Update systems execution
    Update,

    /// Render systems execution
    Render,

    /// Tool execution
    Tool,
}

impl McpFrameworkContext {
    /// Create a new MCP framework context
    pub fn new() -> Self {
        Self {
            inner: FrameworkContext::new(),
            signal_registry: SignalRegistry::new(),
            named_nodes: HashMap::new(),
            perf_history: Vec::new(),
        }
    }

    /// Access the inner framework context (immutable)
    pub fn inner(&self) -> &FrameworkContext {
        &self.inner
    }

    /// Access the inner framework context (mutable)
    pub fn inner_mut(&mut self) -> &mut FrameworkContext {
        &mut self.inner
    }

    /// Access the signal registry (immutable)
    pub fn signal_registry(&self) -> &SignalRegistry {
        &self.signal_registry
    }

    /// Access the signal registry (mutable)
    pub fn signal_registry_mut(&mut self) -> &mut SignalRegistry {
        &mut self.signal_registry
    }

    /// Register a named node for easy lookup
    pub fn register_node(&mut self, name: String, node_id: NodeId) {
        self.named_nodes.insert(name, node_id);
    }

    /// Get a node by name
    pub fn get_named_node(&self, name: &str) -> Option<NodeId> {
        self.named_nodes.get(name).copied()
    }

    /// List all named nodes
    pub fn list_named_nodes(&self) -> Vec<(String, NodeId)> {
        self.named_nodes
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    }

    /// Access the scene (immutable)
    pub fn scene(&self) -> &Scene {
        self.inner.world().resource::<Scene>()
    }

    /// Access the scene (mutable)
    ///
    /// # Safety
    ///
    /// This requires mutable access to the world. Use with caution.
    pub fn scene_mut(&mut self) -> &mut Scene {
        self.inner.world_mut().resource_mut::<Scene>().into_inner()
    }

    /// Run update systems with performance tracking
    pub fn update(&mut self) -> Duration {
        let start = Instant::now();
        self.inner.update();
        let duration = start.elapsed();

        self.perf_history.push(PerfMeasurement {
            kind: PerfKind::Update,
            duration,
            timestamp: Instant::now(),
        });

        duration
    }

    /// Run render systems with performance tracking
    pub fn render(&mut self) -> (Vec<RectInstance>, Duration) {
        let start = Instant::now();
        let instances = self.inner.render();
        let duration = start.elapsed();

        self.perf_history.push(PerfMeasurement {
            kind: PerfKind::Render,
            duration,
            timestamp: Instant::now(),
        });

        (instances, duration)
    }

    /// Get performance history
    pub fn perf_history(&self) -> &[PerfMeasurement] {
        &self.perf_history
    }

    /// Clear performance history
    pub fn clear_perf_history(&mut self) {
        self.perf_history.clear();
    }

    /// Record a tool execution
    pub fn record_tool_execution(&mut self, duration: Duration) {
        self.perf_history.push(PerfMeasurement {
            kind: PerfKind::Tool,
            duration,
            timestamp: Instant::now(),
        });
    }
}

impl Default for McpFrameworkContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arthropod_ecs::Renderable;
    use render_engine::{NodeContent, SceneNode};

    #[test]
    fn test_create_context() {
        let ctx = McpFrameworkContext::new();
        assert!(ctx.scene().get_node(ctx.scene().root()).is_some());
    }

    #[test]
    fn test_register_named_node() {
        let mut ctx = McpFrameworkContext::new();

        let node_id = {
            let scene = ctx.scene_mut();
            let mut node = SceneNode::new(NodeContent::Rect {
                color: render_engine::Color::RED,
            });
            node.bounds = plat_core::Rect::new(0.0, 0.0, 100.0, 100.0);
            node.visible = true;
            node.opacity = 1.0;
            scene.add_node(scene.root(), node)
        };

        ctx.register_node("test_rect".to_string(), node_id);

        assert_eq!(ctx.get_named_node("test_rect"), Some(node_id));
        assert_eq!(ctx.get_named_node("nonexistent"), None);
    }

    #[test]
    fn test_update_with_perf_tracking() {
        let mut ctx = McpFrameworkContext::new();

        // Spawn a test entity
        let node_id = ctx.scene().root();
        ctx.inner_mut().spawn(node_id).insert(Renderable);

        // Run update
        let duration = ctx.update();

        // Verify performance was recorded
        assert_eq!(ctx.perf_history().len(), 1);
        assert_eq!(ctx.perf_history()[0].kind, PerfKind::Update);
        assert!(duration.as_micros() < 10000); // Should be fast
    }

    #[test]
    fn test_render_with_perf_tracking() {
        let mut ctx = McpFrameworkContext::new();

        // Create a visible node with content
        let node_id = {
            let scene = ctx.scene_mut();
            let mut node = SceneNode::new(NodeContent::Rect {
                color: render_engine::Color::RED,
            });
            node.bounds = plat_core::Rect::new(0.0, 0.0, 100.0, 100.0);
            node.visible = true;
            scene.add_node(scene.root(), node)
        };

        ctx.inner_mut().spawn(node_id).insert(Renderable);

        // Must update() first since render collection happens in update schedule
        ctx.update();
        let (instances, duration) = ctx.render();

        assert_eq!(ctx.perf_history().len(), 2);
        assert_eq!(ctx.perf_history()[0].kind, PerfKind::Update);
        assert_eq!(ctx.perf_history()[1].kind, PerfKind::Render);
        assert!(!instances.is_empty());
        assert!(duration.as_micros() < 10000);
    }

    #[test]
    fn test_list_named_nodes() {
        let mut ctx = McpFrameworkContext::new();

        let node1 = {
            let scene = ctx.scene_mut();
            let node = SceneNode::new(NodeContent::Rect {
                color: render_engine::Color::RED,
            });
            scene.add_node(scene.root(), node)
        };

        let node2 = {
            let scene = ctx.scene_mut();
            let node = SceneNode::new(NodeContent::Rect {
                color: render_engine::Color::BLUE,
            });
            scene.add_node(scene.root(), node)
        };

        ctx.register_node("rect1".to_string(), node1);
        ctx.register_node("rect2".to_string(), node2);

        let named = ctx.list_named_nodes();
        assert_eq!(named.len(), 2);
    }
}
