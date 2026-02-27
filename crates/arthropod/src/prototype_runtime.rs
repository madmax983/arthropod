use std::collections::{HashMap, HashSet};

use render_engine::NodeId;
use thiserror::Error;

use crate::figma::{
    ImportedFigmaDocument, PrototypeActionKind, PrototypeEdge, PrototypeOverlayConfig,
    PrototypeTransition, PrototypeTrigger,
};

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PrototypeRuntimeError {
    #[error("cannot initialize runtime: imported document has no mapped nodes")]
    EmptyDocument,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrototypeRuntimeEvent {
    Click { node: NodeId },
    Hover { node: NodeId },
    Drag { node: NodeId },
    Press { node: NodeId },
    KeyDown { node: NodeId },
    Tick { elapsed_ms: u32 },
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrototypeRuntimeEffect {
    Navigate {
        from: NodeId,
        to: NodeId,
        transition: Option<PrototypeTransition>,
        preserve_scroll_position: bool,
    },
    ScrollTo {
        from: NodeId,
        to: NodeId,
        transition: Option<PrototypeTransition>,
    },
    OpenOverlay {
        from: NodeId,
        to: NodeId,
        config: Option<PrototypeOverlayConfig>,
        transition: Option<PrototypeTransition>,
    },
    SwapOverlay {
        from: NodeId,
        to: NodeId,
        config: Option<PrototypeOverlayConfig>,
        transition: Option<PrototypeTransition>,
    },
    CloseOverlay {
        closed: Option<NodeId>,
    },
    Back {
        from: NodeId,
        to: Option<NodeId>,
    },
    OpenUrl {
        url: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverlayState {
    pub node: NodeId,
    pub config: Option<PrototypeOverlayConfig>,
}

/// Executes imported Figma prototype graphs against runtime events.
///
/// The runtime is deterministic and single-threaded:
/// - trigger events evaluate all matching edges in import order
/// - timeout edges accumulate elapsed ms and fire once per active context
/// - navigation updates screen/history and clears overlays
/// - overlay actions manage a LIFO stack
pub struct PrototypeRuntime {
    edges: Vec<PrototypeEdge>,
    figma_to_scene: HashMap<String, NodeId>,
    current_screen: Option<NodeId>,
    history: Vec<NodeId>,
    overlay_stack: Vec<OverlayState>,
    timeout_elapsed_ms: HashMap<usize, u32>,
    timeout_fired: HashSet<usize>,
}

impl PrototypeRuntime {
    pub fn new(imported: &ImportedFigmaDocument) -> Result<Self, PrototypeRuntimeError> {
        let mut root = imported
            .figma_to_scene
            .values()
            .copied()
            .collect::<Vec<_>>();
        root.sort_by_key(|id| id.0);
        let current_screen = root.first().copied();
        if current_screen.is_none() {
            return Err(PrototypeRuntimeError::EmptyDocument);
        }

        Ok(Self {
            edges: imported.prototype_graph.edges.clone(),
            figma_to_scene: imported.figma_to_scene.clone(),
            current_screen,
            history: Vec::new(),
            overlay_stack: Vec::new(),
            timeout_elapsed_ms: HashMap::new(),
            timeout_fired: HashSet::new(),
        })
    }

    pub fn set_current_screen(&mut self, node: NodeId) {
        self.current_screen = Some(node);
        self.reset_timers();
    }

    pub fn current_screen(&self) -> Option<NodeId> {
        self.current_screen
    }

    pub fn history(&self) -> &[NodeId] {
        &self.history
    }

    pub fn overlay_stack(&self) -> &[OverlayState] {
        &self.overlay_stack
    }

    pub fn dispatch(&mut self, event: PrototypeRuntimeEvent) -> Vec<PrototypeRuntimeEffect> {
        match event {
            PrototypeRuntimeEvent::Tick { elapsed_ms } => self.dispatch_tick(elapsed_ms),
            PrototypeRuntimeEvent::Click { node } => {
                self.dispatch_trigger(node, PrototypeTrigger::OnClick)
            }
            PrototypeRuntimeEvent::Hover { node } => {
                self.dispatch_trigger(node, PrototypeTrigger::OnHover)
            }
            PrototypeRuntimeEvent::Drag { node } => {
                self.dispatch_trigger(node, PrototypeTrigger::OnDrag)
            }
            PrototypeRuntimeEvent::Press { node } => {
                self.dispatch_trigger(node, PrototypeTrigger::OnPress)
            }
            PrototypeRuntimeEvent::KeyDown { node } => {
                self.dispatch_trigger(node, PrototypeTrigger::OnKeyDown)
            }
        }
    }

    fn dispatch_tick(&mut self, elapsed_ms: u32) -> Vec<PrototypeRuntimeEffect> {
        let Some(active_node) = self.active_timeout_node() else {
            return Vec::new();
        };
        let mut effects = Vec::new();
        let mut pending = Vec::new();

        for (index, edge) in self.edges.iter().enumerate() {
            if edge.from != active_node || edge.trigger != PrototypeTrigger::AfterTimeout {
                continue;
            }
            if self.timeout_fired.contains(&index) {
                continue;
            }
            let budget = edge.trigger_timeout_ms.unwrap_or(0);
            let total = self
                .timeout_elapsed_ms
                .entry(index)
                .and_modify(|value| *value = value.saturating_add(elapsed_ms))
                .or_insert(elapsed_ms);
            if *total >= budget {
                pending.push(index);
            }
        }

        for index in pending {
            self.timeout_fired.insert(index);
            let edge = self.edges[index].clone();
            effects.extend(self.execute_edge(edge));
        }

        effects
    }

    fn dispatch_trigger(
        &mut self,
        node: NodeId,
        trigger: PrototypeTrigger,
    ) -> Vec<PrototypeRuntimeEffect> {
        let edges = self
            .edges
            .iter()
            .filter(|edge| edge.from == node && edge.trigger == trigger)
            .cloned()
            .collect::<Vec<_>>();

        let mut effects = Vec::new();
        for edge in edges {
            effects.extend(self.execute_edge(edge));
        }
        effects
    }

    fn execute_edge(&mut self, edge: PrototypeEdge) -> Vec<PrototypeRuntimeEffect> {
        let mut effects = Vec::new();
        let dest = edge
            .to_figma_id
            .as_ref()
            .and_then(|id| self.figma_to_scene.get(id))
            .copied();

        match edge.action {
            PrototypeActionKind::Navigate => {
                if let Some(to) = dest {
                    self.push_history(edge.from);
                    self.current_screen = Some(to);
                    self.overlay_stack.clear();
                    self.reset_timers();
                    effects.push(PrototypeRuntimeEffect::Navigate {
                        from: edge.from,
                        to,
                        transition: edge.transition,
                        preserve_scroll_position: edge.preserve_scroll_position,
                    });
                }
            }
            PrototypeActionKind::ScrollTo => {
                if let Some(to) = dest {
                    effects.push(PrototypeRuntimeEffect::ScrollTo {
                        from: edge.from,
                        to,
                        transition: edge.transition,
                    });
                }
            }
            PrototypeActionKind::OpenOverlay => {
                if let Some(to) = dest {
                    self.overlay_stack.push(OverlayState {
                        node: to,
                        config: edge.overlay.clone(),
                    });
                    self.reset_timers();
                    effects.push(PrototypeRuntimeEffect::OpenOverlay {
                        from: edge.from,
                        to,
                        config: edge.overlay,
                        transition: edge.transition,
                    });
                }
            }
            PrototypeActionKind::SwapOverlay => {
                if let Some(to) = dest {
                    if let Some(top) = self.overlay_stack.last_mut() {
                        top.node = to;
                        top.config = edge.overlay.clone();
                    } else {
                        self.overlay_stack.push(OverlayState {
                            node: to,
                            config: edge.overlay.clone(),
                        });
                    }
                    self.reset_timers();
                    effects.push(PrototypeRuntimeEffect::SwapOverlay {
                        from: edge.from,
                        to,
                        config: edge.overlay,
                        transition: edge.transition,
                    });
                }
            }
            PrototypeActionKind::CloseOverlay => {
                let closed = self.overlay_stack.pop().map(|entry| entry.node);
                self.reset_timers();
                effects.push(PrototypeRuntimeEffect::CloseOverlay { closed });
            }
            PrototypeActionKind::Back => {
                if let Some(closed) = self.overlay_stack.pop() {
                    self.reset_timers();
                    effects.push(PrototypeRuntimeEffect::CloseOverlay {
                        closed: Some(closed.node),
                    });
                    effects.push(PrototypeRuntimeEffect::Back {
                        from: edge.from,
                        to: self.current_screen,
                    });
                } else {
                    let target = self.history.pop();
                    if let Some(node) = target {
                        self.current_screen = Some(node);
                    }
                    self.reset_timers();
                    effects.push(PrototypeRuntimeEffect::Back {
                        from: edge.from,
                        to: target,
                    });
                }
            }
            PrototypeActionKind::Url => {
                if let Some(url) = edge.url {
                    effects.push(PrototypeRuntimeEffect::OpenUrl { url });
                }
            }
            PrototypeActionKind::Unknown => {}
        }

        effects
    }

    fn push_history(&mut self, from: NodeId) {
        if self.current_screen.is_some_and(|current| current != from) {
            self.history.push(from);
            return;
        }
        if self.current_screen == Some(from) {
            self.history.push(from);
        }
    }

    fn active_timeout_node(&self) -> Option<NodeId> {
        self.overlay_stack
            .last()
            .map(|overlay| overlay.node)
            .or(self.current_screen)
    }

    fn reset_timers(&mut self) {
        self.timeout_elapsed_ms.clear();
        self.timeout_fired.clear();
    }
}
