//! Signal primitive - the atomic unit of reactive state.
//!
//! Signals are the primary way to store state in `flux-state`.
//! They are thread-safe containers that notify subscribers (Effects or Computed values)
//! whenever their content changes.

use crate::runtime::{NodeId, Runtime};
use std::sync::{Arc, Mutex};

/// A reactive signal - the atomic unit of state.
///
/// Signals hold a value and notify dependents when that value changes.
///
/// # Example
///
/// ```
/// use flux_state::{Runtime, Signal};
///
/// let runtime = Runtime::new();
/// let count = Signal::new(runtime, 0);
/// ```
pub struct Signal<T> {
    id: NodeId,
    runtime: Arc<Runtime>,
    _marker: std::marker::PhantomData<T>,
}

/// Read-only view of a signal.
///
/// Use this to access the signal's value in Effects and Computed closures.
/// Reading from a `ReadSignal` automatically tracks dependencies.
#[derive(Clone)]
pub struct ReadSignal<T> {
    id: NodeId,
    runtime: Arc<Runtime>,
    _marker: std::marker::PhantomData<T>,
}

/// Write-only view of a signal.
///
/// Use this to update the signal's value. Updating a signal triggers
/// updates in all dependent Effects and Computed values.
#[derive(Clone)]
pub struct WriteSignal<T> {
    id: NodeId,
    runtime: Arc<Runtime>,
    _marker: std::marker::PhantomData<T>,
}

// SAFETY: Signal uses internal Mutex locking via Runtime, so it is safe to share
// between threads even if T is !Sync (e.g., RefCell), as long as T is Send.
// T must be Send because it is stored in Box<dyn Any + Send>.
unsafe impl<T: Send> Sync for Signal<T> {}
unsafe impl<T: Send> Sync for ReadSignal<T> {}
unsafe impl<T: Send> Sync for WriteSignal<T> {}

impl<T: 'static + Send> Signal<T> {
    /// Create a new signal with an initial value.
    ///
    /// # Example
    ///
    /// ```
    /// # use flux_state::{Runtime, Signal};
    /// let runtime = Runtime::new();
    /// let count = Signal::new(runtime, 100);
    /// ```
    pub fn new(runtime: Arc<Runtime>, value: T) -> Self {
        let id = runtime.create_signal(Box::new(Mutex::new(value)));
        Self {
            id,
            runtime,
            _marker: std::marker::PhantomData,
        }
    }

    /// Split into read and write handles.
    ///
    /// This is useful when you want to pass read access to some components
    /// and write access to others, or when capturing in closures.
    ///
    /// # Example
    ///
    /// ```
    /// # use flux_state::{Runtime, Signal};
    /// # let runtime = Runtime::new();
    /// let count = Signal::new(runtime, 0);
    /// let (read, write) = count.split();
    /// ```
    pub fn split(self) -> (ReadSignal<T>, WriteSignal<T>) {
        (
            ReadSignal {
                id: self.id,
                runtime: Arc::clone(&self.runtime),
                _marker: std::marker::PhantomData,
            },
            WriteSignal {
                id: self.id,
                runtime: Arc::clone(&self.runtime),
                _marker: std::marker::PhantomData,
            },
        )
    }
}

impl<T: Clone + 'static + Send> ReadSignal<T> {
    /// Get the current value and track this dependency.
    ///
    /// Call this inside an Effect or Computed closure to subscribe to updates.
    ///
    /// # Example
    ///
    /// ```
    /// # use flux_state::{Runtime, Signal, Effect};
    /// # let runtime = Runtime::new();
    /// let count = Signal::new(runtime.clone(), 0);
    /// let (read, _) = count.split();
    ///
    /// Effect::new(runtime, move || {
    ///     println!("Count is: {}", read.get()); // Auto-subscribes
    /// });
    /// ```
    pub fn get(&self) -> T {
        self.runtime.track(self.id);
        self.runtime
            .with_signal_value(self.id, |v: &dyn std::any::Any| {
                v.downcast_ref::<Mutex<T>>()
                    .expect("Type mismatch")
                    .lock()
                    .unwrap()
                    .clone()
            })
    }

    /// Get the current value without tracking dependencies.
    ///
    /// Use this when you want to read the value without causing the Effect
    /// to re-run when this signal changes.
    ///
    /// # Example
    ///
    /// ```
    /// # use flux_state::{Runtime, Signal, Effect};
    /// # let runtime = Runtime::new();
    /// let count = Signal::new(runtime.clone(), 0);
    /// let (read, _) = count.split();
    ///
    /// Effect::new(runtime, move || {
    ///     // This effect will NOT re-run when count changes
    ///     println!("Count is: {}", read.get_untracked());
    /// });
    /// ```
    pub fn get_untracked(&self) -> T {
        self.runtime
            .with_signal_value(self.id, |v: &dyn std::any::Any| {
                v.downcast_ref::<Mutex<T>>()
                    .expect("Type mismatch")
                    .lock()
                    .unwrap()
                    .clone()
            })
    }
}

impl<T: 'static + Send> WriteSignal<T> {
    /// Set a new value and notify dependents.
    ///
    /// # Example
    ///
    /// ```
    /// # use flux_state::{Runtime, Signal};
    /// # let runtime = Runtime::new();
    /// let count = Signal::new(runtime, 0);
    /// let (_, write) = count.split();
    ///
    /// write.set(42);
    /// ```
    pub fn set(&self, value: T) {
        self.runtime
            .with_signal_value(self.id, |v: &dyn std::any::Any| {
                *v.downcast_ref::<Mutex<T>>()
                    .expect("Type mismatch")
                    .lock()
                    .unwrap() = value;
            });
        self.runtime.notify(self.id);
    }

    /// Update the value with a function and notify dependents.
    ///
    /// # Example
    ///
    /// ```
    /// # use flux_state::{Runtime, Signal};
    /// # let runtime = Runtime::new();
    /// let count = Signal::new(runtime, 0);
    /// let (_, write) = count.split();
    ///
    /// write.update(|c| *c += 1);
    /// ```
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        self.runtime
            .with_signal_value(self.id, |v: &dyn std::any::Any| {
                f(&mut v
                    .downcast_ref::<Mutex<T>>()
                    .expect("Type mismatch")
                    .lock()
                    .unwrap());
            });
        self.runtime.notify(self.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::Runtime;
    use std::thread;

    #[test]
    fn test_signal_basic() {
        let runtime = Runtime::new();
        let signal = Signal::new(Arc::clone(&runtime), 0);
        let (read, write) = signal.split();

        assert_eq!(read.get(), 0);

        write.set(10);
        assert_eq!(read.get(), 10);

        write.update(|v| *v += 5);
        assert_eq!(read.get(), 15);
    }

    #[test]
    fn test_get_untracked() {
        let runtime = Runtime::new();
        let signal = Signal::new(Arc::clone(&runtime), 1);
        let (read, _write) = signal.split();
        assert_eq!(read.get_untracked(), 1);
    }

    #[test]
    fn test_threading() {
        let runtime = Runtime::new();
        let signal = Signal::new(Arc::clone(&runtime), 0);
        let (read, write) = signal.split();

        let w_thread = write.clone();
        let handle_write = thread::spawn(move || {
            w_thread.set(99);
        });

        handle_write.join().unwrap();

        let r_thread = read.clone();
        let handle_read = thread::spawn(move || r_thread.get());

        let val = handle_read.join().unwrap();

        assert_eq!(val, 99);
        assert_eq!(read.get(), 99);
    }
}
