use flux_state::{ReadSignal, Runtime, Signal, WriteSignal};
use std::sync::{Arc, Mutex};

/// A timeline managing the history of state changes.
///
/// It supports undo/redo operations for all registered `RetroSignal`s.
pub struct Timeline {
    past: Vec<Transaction>,
    future: Vec<Transaction>,
    /// Operations currently being batched (optional feature, for now we can commit immediately)
    current_batch: Option<Transaction>,
}

struct Transaction {
    ops: Vec<Operation>,
}

struct Operation {
    undo: Box<dyn Fn() + Send + Sync>,
    redo: Box<dyn Fn() + Send + Sync>,
}

impl Timeline {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            past: Vec::new(),
            future: Vec::new(),
            current_batch: None,
        }))
    }

    /// Undo the last transaction.
    pub fn undo(&mut self) {
        if let Some(tx) = self.past.pop() {
            // Execute undo operations in reverse order
            for op in tx.ops.iter().rev() {
                (op.undo)();
            }
            self.future.push(tx);
        }
    }

    /// Redo the last undone transaction.
    pub fn redo(&mut self) {
        if let Some(tx) = self.future.pop() {
            // Execute redo operations in original order
            for op in tx.ops.iter() {
                (op.redo)();
            }
            self.past.push(tx);
        }
    }

    /// Record a change operation.
    fn record(&mut self, op: Operation) {
        // Clearing future because we are branching off
        if !self.future.is_empty() {
            self.future.clear();
        }

        if let Some(batch) = &mut self.current_batch {
            batch.ops.push(op);
        } else {
            self.past.push(Transaction { ops: vec![op] });
        }
    }

    /// Start a transaction batch.
    pub fn begin_transaction(&mut self) {
        if self.current_batch.is_none() {
            self.current_batch = Some(Transaction { ops: Vec::new() });
        }
    }

    /// Commit the current transaction batch.
    pub fn commit_transaction(&mut self) {
        if let Some(batch) = self.current_batch.take() {
            if !batch.ops.is_empty() {
                self.past.push(batch);
            }
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
}

impl<T: Clone + 'static + Send + Sync> RetroSignal<T> {
    pub fn new(runtime: Arc<Runtime>, timeline: Arc<Mutex<Timeline>>, value: T) -> Self {
        let inner = Signal::new(runtime, value);
        let (read, write) = inner.split();
        Self {
            read,
            write,
            timeline,
        }
    }

    /// Set the value and record it in history.
    pub fn set(&self, new_value: T) {
        let old_value = self.read.get_untracked();

        // Prepare undo/redo closures
        // We use WriteSignal for updates inside the closures to trigger reactivity
        let write_undo = self.write.clone();
        let val_undo = old_value.clone();

        let write_redo = self.write.clone();
        let val_redo = new_value.clone();

        let op = Operation {
            undo: Box::new(move || write_undo.set(val_undo.clone())),
            redo: Box::new(move || write_redo.set(val_redo.clone())),
        };

        // Record to timeline
        self.timeline.lock().unwrap().record(op);

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
        let signal = RetroSignal::new(runtime, timeline, 0);

        assert_eq!(signal.get(), 0);

        signal.set(10);
        assert_eq!(signal.get(), 10);
    }

    #[test]
    fn test_undo_redo() {
        let runtime = Runtime::new();
        let timeline = Timeline::new();
        let signal = RetroSignal::new(runtime, timeline.clone(), 0);

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
        let signal = RetroSignal::new(runtime, timeline.clone(), 0);

        signal.set(10);
        signal.set(20);

        // Undo 20 -> 10
        timeline.lock().unwrap().undo();
        assert_eq!(signal.get(), 10);

        // New future: 10 -> 30
        signal.set(30);
        assert_eq!(signal.get(), 30);

        // Attempt redo (should fail/do nothing as future is cleared)
        timeline.lock().unwrap().redo();
        assert_eq!(signal.get(), 30);

        // Undo 30 -> 10
        timeline.lock().unwrap().undo();
        assert_eq!(signal.get(), 10);
    }
}
