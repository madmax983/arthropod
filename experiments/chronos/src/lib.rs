//! # Chronos
//!
//! Experimental time-travel and transaction history tracking for reactive signals.
//!
//! This crate implements the `Timeline` and `RetroSignal` primitives, enabling complex
//! state interactions like undo/redo, timeline jumping, and transaction batching
//! directly on top of `flux-state` reactive properties.
//!
//! ## Core Concepts
//!
//! - **Timeline**: A shared history state that tracks transactions as a tree.
//! - **RetroSignal**: A wrapper around `flux-state::Signal` that automatically records
//!   its changes to a `Timeline`.

use flux_state::{ReadSignal, Runtime, Signal, WriteSignal};
use std::sync::{Arc, Mutex};

/// A timeline managing the history of state changes.
///
/// It supports undo/redo operations for all registered `RetroSignal`s and branching history.
pub struct Timeline {
    nodes: Vec<Node>,
    current_node: usize,
    /// Operations currently being batched
    current_batch: Option<Transaction>,
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
    /// Create a new timeline with a root node.
    ///
    /// ## Examples
    ///
    /// ```
    /// use chronos::Timeline;
    ///
    /// let timeline = Timeline::new();
    /// ```
    pub fn new() -> Arc<Mutex<Self>> {
        // Create root node with empty transaction
        let root = Node {
            parent: None,
            children: Vec::new(),
            transaction: Transaction {
                description: "Root".to_string(),
                ops: Vec::new(),
            },
        };

        Arc::new(Mutex::new(Self {
            nodes: vec![root],
            current_node: 0,
            current_batch: None,
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
        }
    }

    /// Record a change operation.
    fn record(&mut self, description: String, op: Operation) {
        if let Some(batch) = &mut self.current_batch {
            batch.ops.push(op);
            // Append description? batch.description += &description?
            // For now, batch description is set at commit time or init.
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
        if let Some(batch) = self.current_batch.take() {
            if !batch.ops.is_empty() {
                self.add_node(batch);
            }
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
    }

    /// Print the transaction tree to standard output for debugging.
    ///
    /// ## Examples
    ///
    /// ```
    /// use chronos::Timeline;
    ///
    /// let timeline = Timeline::new();
    /// timeline.lock().unwrap().print_tree();
    /// ```
    pub fn print_tree(&self) {
        self.print_node(0, 0);
    }

    fn print_node(&self, node_idx: usize, depth: usize) {
        if node_idx >= self.nodes.len() {
            return;
        }
        let node = &self.nodes[node_idx];
        let indent = "  ".repeat(depth);
        let marker = if node_idx == self.current_node {
            "*"
        } else {
            " "
        };
        println!(
            "{}{}[{}] {}",
            indent, marker, node_idx, node.transaction.description
        );

        for &child in &node.children {
            self.print_node(child, depth + 1);
        }
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
    /// Create a new retro signal that records its changes to the provided timeline.
    ///
    /// ## Examples
    ///
    /// ```
    /// use flux_state::Runtime;
    /// use chronos::{Timeline, RetroSignal};
    ///
    /// let runtime = Runtime::new();
    /// let timeline = Timeline::new();
    /// let signal = RetroSignal::new(runtime, timeline, "test_sig", 0);
    /// assert_eq!(signal.get(), 0);
    /// ```
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

        self.timeline.lock().unwrap().record(description, op);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retro_signal_basic() {
        let runtime = Runtime::new();
        let timeline = Timeline::new();
        let signal = RetroSignal::new(runtime, timeline, "test_sig", 0);

        assert_eq!(signal.get(), 0);

        signal.set(10);
        assert_eq!(signal.get(), 10);
    }

    #[test]
    fn test_undo_redo() {
        let runtime = Runtime::new();
        let timeline = Timeline::new();
        let signal = RetroSignal::new(runtime, timeline.clone(), "test_sig", 0);

        signal.set(10);
        signal.set(20);

        assert_eq!(signal.get(), 20);

        // Undo 20 -> 10
        timeline.lock().unwrap().undo();
        assert_eq!(signal.get(), 10);

        // Undo 10 -> 0
        timeline.lock().unwrap().undo();
        assert_eq!(signal.get(), 0);

        // Redo 0 -> 10
        timeline.lock().unwrap().redo();
        assert_eq!(signal.get(), 10);

        // Redo 10 -> 20
        timeline.lock().unwrap().redo();
        assert_eq!(signal.get(), 20);
    }

    #[test]
    fn test_branching_history() {
        let runtime = Runtime::new();
        let timeline = Timeline::new();
        let signal = RetroSignal::new(runtime, timeline.clone(), "test_sig", 0);

        signal.set(10);
        signal.set(20);

        // Undo 20 -> 10
        timeline.lock().unwrap().undo();
        assert_eq!(signal.get(), 10);

        // New future: 10 -> 30 (Branch created)
        signal.set(30);
        assert_eq!(signal.get(), 30);

        // Undo 30 -> 10
        timeline.lock().unwrap().undo();
        assert_eq!(signal.get(), 10);

        // Redo should go to 30 (last active branch)
        timeline.lock().unwrap().redo();
        assert_eq!(signal.get(), 30);
    }
}

#[test]
fn test_batch_transaction() {
    let runtime = Runtime::new();
    let timeline = Timeline::new();
    let signal1 = RetroSignal::new(runtime.clone(), timeline.clone(), "sig1", 0);
    let signal2 = RetroSignal::new(runtime.clone(), timeline.clone(), "sig2", 0);

    timeline
        .lock()
        .unwrap()
        .begin_transaction("Batch Update".to_string());

    signal1.set(10);
    signal2.set(20);

    timeline.lock().unwrap().commit_transaction();

    assert_eq!(signal1.get(), 10);
    assert_eq!(signal2.get(), 20);

    // One undo should revert both changes
    timeline.lock().unwrap().undo();
    assert_eq!(signal1.get(), 0);
    assert_eq!(signal2.get(), 0);

    // One redo should re-apply both changes
    timeline.lock().unwrap().redo();
    assert_eq!(signal1.get(), 10);
    assert_eq!(signal2.get(), 20);
}

#[test]
fn test_jump_to() {
    let runtime = Runtime::new();
    let timeline = Timeline::new();
    let signal = RetroSignal::new(runtime.clone(), timeline.clone(), "sig", 0);

    signal.set(10); // Node 1
    signal.set(20); // Node 2

    timeline.lock().unwrap().undo(); // Back to Node 1
    signal.set(30); // Node 3 (Branch)

    // Currently at Node 3, value is 30.
    assert_eq!(signal.get(), 30);

    // Jump to Node 2 (value 20)
    timeline.lock().unwrap().jump_to(2);
    assert_eq!(signal.get(), 20);

    // Jump to Node 3 (value 30)
    timeline.lock().unwrap().jump_to(3);
    assert_eq!(signal.get(), 30);

    // Jump to Node 1 (value 10)
    timeline.lock().unwrap().jump_to(1);
    assert_eq!(signal.get(), 10);
}
