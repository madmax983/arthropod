use flux_state::{ReadSignal, Runtime, Signal, WriteSignal};
use render_engine::{Color, NodeId};
use std::sync::{Arc, Mutex};
use widget_core::{Button, Column, Text, Widget, WidgetContext};

/// A timeline managing the history of state changes.
///
/// It supports undo/redo operations for all registered `RetroSignal`s and branching history.
pub struct Timeline {
    nodes: Vec<Node>,
    current_node: usize,
    /// Operations currently being batched
    current_batch: Option<Transaction>,
    /// Read handle for version signal
    version_read: ReadSignal<usize>,
    /// Write handle for version signal
    version_write: WriteSignal<usize>,
}

struct Node {
    parent: Option<usize>,
    children: Vec<usize>,
    transaction: Transaction,
}

struct Transaction {
    description: String,
    ops: Vec<Operation>,
}

struct Operation {
    undo: Box<dyn Fn() + Send + Sync>,
    redo: Box<dyn Fn() + Send + Sync>,
}

impl Timeline {
    pub fn new(runtime: Arc<Runtime>) -> Arc<Mutex<Self>> {
        // Create root node with empty transaction
        let root = Node {
            parent: None,
            children: Vec::new(),
            transaction: Transaction {
                description: "Initial State".to_string(),
                ops: Vec::new(),
            },
        };

        let version = Signal::new(runtime, 0);
        let (version_read, version_write) = version.split();

        Arc::new(Mutex::new(Self {
            nodes: vec![root],
            current_node: 0,
            current_batch: None,
            version_read,
            version_write,
        }))
    }

    /// Undo the last transaction (move to parent).
    pub fn undo(&mut self) {
        if let Some(parent_idx) = self.nodes[self.current_node].parent {
            // Execute undo operations in reverse order
            let tx = &self.nodes[self.current_node].transaction;
            for op in tx.ops.iter().rev() {
                (op.undo)();
            }
            self.current_node = parent_idx;
            self.notify();
        }
    }

    /// Redo the last undone transaction (move to last child).
    pub fn redo(&mut self) {
        if let Some(&child_idx) = self.nodes[self.current_node].children.last() {
            // Execute redo operations in original order
            let tx = &self.nodes[child_idx].transaction;
            for op in tx.ops.iter() {
                (op.redo)();
            }
            self.current_node = child_idx;
            self.notify();
        }
    }

    /// Record a change operation.
    fn record(&mut self, description: String, op: Operation) {
        if let Some(batch) = &mut self.current_batch {
            batch.ops.push(op);
        } else {
            // Create a new transaction node immediately
            let tx = Transaction {
                description,
                ops: vec![op],
            };
            self.add_node(tx);
        }
    }

    fn add_node(&mut self, tx: Transaction) {
        let new_node_idx = self.nodes.len();
        let new_node = Node {
            parent: Some(self.current_node),
            children: Vec::new(),
            transaction: tx,
        };
        self.nodes.push(new_node);
        self.nodes[self.current_node].children.push(new_node_idx);
        self.current_node = new_node_idx;
        self.notify();
    }

    /// Start a transaction batch.
    pub fn begin_transaction(&mut self, description: String) {
        if self.current_batch.is_none() {
            self.current_batch = Some(Transaction {
                description,
                ops: Vec::new(),
            });
        }
    }

    /// Commit the current transaction batch.
    pub fn commit_transaction(&mut self) {
        if let Some(batch) = self.current_batch.take().filter(|b| !b.ops.is_empty()) {
            self.add_node(batch);
        }
    }

    fn get_path_from_root(&self, target: usize) -> Vec<usize> {
        let mut path = Vec::new();
        let mut curr = target;
        loop {
            path.push(curr);
            if let Some(parent) = self.nodes[curr].parent {
                curr = parent;
            } else {
                break;
            }
        }
        path.reverse();
        path
    }

    /// Jump to a specific node in the timeline, undoing/redoing as necessary.
    pub fn jump_to(&mut self, target: usize) {
        if target >= self.nodes.len() {
            return;
        }

        let current_path = self.get_path_from_root(self.current_node);
        let target_path = self.get_path_from_root(target);

        // Find LCA
        let mut lca_idx = 0;
        let len = std::cmp::min(current_path.len(), target_path.len());
        for i in 0..len {
            if current_path[i] == target_path[i] {
                lca_idx = current_path[i];
            } else {
                break;
            }
        }

        // Undo to LCA
        while self.current_node != lca_idx {
            self.undo();
        }

        // Redo to target
        let lca_pos = target_path.iter().position(|&x| x == lca_idx).unwrap();

        for &next_node in target_path.iter().skip(lca_pos + 1) {
            // Execute redo operations for the specific child path
            let tx = &self.nodes[next_node].transaction;
            for op in tx.ops.iter() {
                (op.redo)();
            }
            self.current_node = next_node;
        }
        self.notify();
    }

    fn notify(&self) {
        self.version_write.set(self.current_node);
    }
}

/// A time-traveling signal wrapper.
///
/// It wraps a `flux_state::Signal` and records all changes to a `Timeline`.
#[derive(Clone)]
pub struct RetroSignal<T> {
    read: ReadSignal<T>,
    write: WriteSignal<T>,
    timeline: Arc<Mutex<Timeline>>,
    name: String,
}

impl<T: Clone + 'static + Send + Sync> RetroSignal<T> {
    pub fn new(
        runtime: Arc<Runtime>,
        timeline: Arc<Mutex<Timeline>>,
        name: impl Into<String>,
        value: T,
    ) -> Self {
        let inner = Signal::new(runtime, value);
        let (read, write) = inner.split();
        Self {
            read,
            write,
            timeline,
            name: name.into(),
        }
    }

    /// Set the value and record it in history.
    pub fn set(&self, new_value: T) {
        let old_value = self.read.get_untracked();

        // Prepare undo/redo closures
        let write_undo = self.write.clone();
        let val_undo = old_value.clone();

        let write_redo = self.write.clone();
        let val_redo = new_value.clone();

        let op = Operation {
            undo: Box::new(move || write_undo.set(val_undo.clone())),
            redo: Box::new(move || write_redo.set(val_redo.clone())),
        };

        // Record to timeline
        let description = format!("Set {}", self.name);

        self.timeline
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .record(description, op);

        // Apply change
        self.write.set(new_value);
    }

    /// Update the value and record it in history.
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        let old_value = self.read.get_untracked();
        let mut new_value = old_value.clone();
        f(&mut new_value);
        self.set(new_value);
    }

    /// Get the read handle.
    pub fn read(&self) -> &ReadSignal<T> {
        &self.read
    }

    /// Get the current value (reactive).
    pub fn get(&self) -> T {
        self.read.get()
    }
}

/// Widget that displays the timeline and allows time travel.
pub struct ChronosDebugger {
    timeline: Arc<Mutex<Timeline>>,
}

impl ChronosDebugger {
    pub fn new(timeline: Arc<Mutex<Timeline>>) -> Self {
        Self { timeline }
    }
}

impl Widget for ChronosDebugger {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        let timeline = self.timeline.clone();

        // Subscribe to version changes to trigger rebuild/updates
        // In a real implementation, we'd use a reactive list or similar.
        // For now, we'll just read the current state.
        // To make it reactive, we need to access the signal inside the lock.
        let (version, nodes) = {
            let tl = timeline
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let version = tl.version_read.get(); // Establish dependency

            // Collect node info for display
            let nodes: Vec<(usize, String, bool)> = tl
                .nodes
                .iter()
                .enumerate()
                .map(|(i, node)| {
                    (
                        i,
                        node.transaction.description.clone(),
                        i == tl.current_node,
                    )
                })
                .collect();

            (version, nodes)
        };

        // Build the UI
        Column::new((
            Text::new(format!("Chronos Timeline (v{})", version))
                .size(16.0)
                .color(Color::WHITE),
            // List of transactions
            // Note: In a real app we should use a proper List widget or scroll area.
            // Here we just stack buttons.
            {
                let timeline = timeline.clone();

                // Simplified approach: Render last 10 entries to avoid overflow
                let start_idx = if nodes.len() > 10 {
                    nodes.len() - 10
                } else {
                    0
                };

                let mut buttons = Vec::new();
                for (idx, desc, is_current) in nodes.iter().skip(start_idx) {
                    let idx = *idx;
                    let desc = desc.clone();
                    let is_current = *is_current;
                    let timeline = timeline.clone();

                    let marker = if is_current { "-> " } else { "   " };
                    let label = format!("{}{}: {}", marker, idx, desc);

                    let btn = Button::new(label).on_click(move || {
                        timeline
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .jump_to(idx);
                    });

                    // Style the current node differently
                    let btn = if is_current { btn.primary() } else { btn };

                    buttons.push(btn);
                }

                // Helper to create a vertical stack of buttons
                // Since `buttons` is Vec<Button>, and Column takes a tuple,
                // we have to use `col` macro or similar if available, or just manually build.
                // WidgetTuple is implemented for Vec? No.
                // But `widget_core::list_from` might be useful if it exists.
                // Checked `prelude`: `list_from` is exported.

                widget_core::list_from(buttons)
            },
        ))
        .gap(5.0)
        .padding(10.0)
        .build(ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retro_signal_basic() {
        let runtime = Runtime::new();
        let timeline = Timeline::new(runtime.clone());
        let signal = RetroSignal::new(runtime, timeline, "test_sig", 0);

        assert_eq!(signal.get(), 0);

        signal.set(10);
        assert_eq!(signal.get(), 10);
    }

    #[test]
    fn test_undo_redo() {
        let runtime = Runtime::new();
        let timeline = Timeline::new(runtime.clone());
        let signal = RetroSignal::new(runtime, timeline.clone(), "test_sig", 0);

        signal.set(10);
        signal.set(20);

        assert_eq!(signal.get(), 20);

        // Undo 20 -> 10
        timeline
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .undo();
        assert_eq!(signal.get(), 10);

        // Undo 10 -> 0
        timeline
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .undo();
        assert_eq!(signal.get(), 0);

        // Redo 0 -> 10
        timeline
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .redo();
        assert_eq!(signal.get(), 10);

        // Redo 10 -> 20
        timeline
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .redo();
        assert_eq!(signal.get(), 20);
    }

    #[test]
    fn test_branching_history() {
        let runtime = Runtime::new();
        let timeline = Timeline::new(runtime.clone());
        let signal = RetroSignal::new(runtime, timeline.clone(), "test_sig", 0);

        signal.set(10);
        signal.set(20);

        // Undo 20 -> 10
        timeline
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .undo();
        assert_eq!(signal.get(), 10);

        // New future: 10 -> 30 (Branch created)
        signal.set(30);
        assert_eq!(signal.get(), 30);

        // Undo 30 -> 10
        timeline
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .undo();
        assert_eq!(signal.get(), 10);

        // Redo should go to 30 (last active branch)
        timeline
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .redo();
        assert_eq!(signal.get(), 30);
    }
}
