//! Signal primitive - the atomic unit of reactive state.
//!
//! Signals are the primary way to store state in `flux-state`.
//! They are thread-safe containers that notify subscribers (Effects or Computed values)
//! whenever their content changes.

use crate::runtime::{NodeId, Runtime};
use std::sync::{Arc, RwLock};

/// A reactive signal - the atomic unit of state.
///
/// Signals hold a value and notify dependents when that value changes.
/// They are the primary input to [`crate::Computed`] values and [`crate::Effect`]s.
///
/// # Example
///
/// ```
/// use flux_state::{Runtime, Signal};
///
/// let runtime = Runtime::new();
/// let count = Signal::new(runtime.clone(), 0);
///
/// // Typically, you immediately split the signal into read/write handles
/// let (read, write) = count.split();
/// ```
///
/// # See Also
///
/// - [`crate::Computed`]: Derive state from signals automatically.
/// - [`crate::Effect`]: Run side effects when signals change.
#[derive(Clone)]
pub struct Signal<T> {
    id: NodeId,
    runtime: Arc<Runtime>,
    handle: Arc<RwLock<T>>,
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
    handle: Arc<RwLock<T>>,
    is_computed: bool,
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
    handle: Arc<RwLock<T>>,
    _marker: std::marker::PhantomData<T>,
}

// NOTE: We rely on auto-traits for Sync.
// Signal<T> is Sync if T is Sync (because RwLock<T> is Sync if T is Sync).
// This prevents sharing !Sync types (like RefCell) across threads, which avoids data races.

impl<T: std::fmt::Debug + 'static + Send + Sync> std::fmt::Debug for Signal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Track the dependency so that effects re-run when the signal changes.
        self.runtime.track(self.id);

        match self.handle.try_read() {
            Ok(val) => write!(f, "Signal(id: {:?}, value: {:?})", self.id, *val),
            Err(_) => write!(f, "Signal(id: {:?}, value: <locked>)", self.id),
        }
    }
}

impl<T: std::fmt::Debug + 'static + Send + Sync> std::fmt::Debug for ReadSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Track the dependency so that effects re-run when the signal changes.
        self.runtime.track(self.id);

        match self.handle.try_read() {
            Ok(val) => write!(f, "ReadSignal(id: {:?}, value: {:?})", self.id, *val),
            Err(_) => write!(f, "ReadSignal(id: {:?}, value: <locked>)", self.id),
        }
    }
}

impl<T: std::fmt::Debug + 'static + Send + Sync> std::fmt::Debug for WriteSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Track the dependency so that effects re-run when the signal changes.
        self.runtime.track(self.id);

        match self.handle.try_read() {
            Ok(val) => write!(f, "WriteSignal(id: {:?}, value: {:?})", self.id, *val),
            Err(_) => write!(f, "WriteSignal(id: {:?}, value: <locked>)", self.id),
        }
    }
}

impl<T: 'static + Send + Sync> Signal<T> {
    /// Create a new signal with an initial value.
    ///
    /// # Example
    ///
    /// ```
    /// # use flux_state::{Runtime, Signal};
    /// let runtime = Runtime::new();
    /// let count = Signal::new(runtime, 100);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the runtime fails to allocate a new signal ID (unlikely).
    pub fn new(runtime: Arc<Runtime>, value: T) -> Self {
        let handle = Arc::new(RwLock::new(value));
        let id = runtime.create_signal(handle.clone());
        Self {
            id,
            runtime,
            handle,
            _marker: std::marker::PhantomData,
        }
    }

    /// Split into read and write handles.
    ///
    /// This is useful when you want to pass read access to some components
    /// and write access to others, or when capturing in closures.
    ///
    /// **Note:** This method consumes the `Signal`. If you need to keep the original
    /// signal handle, you should clone it first.
    ///
    /// # Example
    ///
    /// ```
    /// # use flux_state::{Runtime, Signal};
    /// # let runtime = Runtime::new();
    /// let count = Signal::new(runtime, 0);
    ///
    /// // Standard usage: split and consume the original signal
    /// let (read, write) = count.split();
    /// ```
    ///
    /// ```
    /// # use flux_state::{Runtime, Signal};
    /// # let runtime = Runtime::new();
    /// let count = Signal::new(runtime, 0);
    ///
    /// // Keep the original signal by cloning
    /// let (read, write) = count.clone().split();
    /// ```
    pub fn split(self) -> (ReadSignal<T>, WriteSignal<T>) {
        (
            ReadSignal {
                id: self.id,
                runtime: Arc::clone(&self.runtime),
                handle: Arc::clone(&self.handle),
                is_computed: false,
                _marker: std::marker::PhantomData,
            },
            WriteSignal {
                id: self.id,
                runtime: Arc::clone(&self.runtime),
                handle: Arc::clone(&self.handle),
                _marker: std::marker::PhantomData,
            },
        )
    }

    /// Access the signal value safely with a closure.
    ///
    /// This method allows accessing the value without cloning it.
    /// It automatically tracks dependencies.
    ///
    /// # Example
    ///
    /// ```
    /// # use flux_state::{Runtime, Signal};
    /// # let runtime = Runtime::new();
    /// let count = Signal::new(runtime, vec![1, 2, 3]);
    ///
    /// // Access length without cloning the vector
    /// let len = count.with(|v| v.len());
    /// assert_eq!(len, 3);
    /// ```
    ///
    /// # Panics
    ///
    /// - Panics if the internal lock is poisoned.
    ///
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.runtime.track(self.id);
        let guard = self
            .handle
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        f(&*guard)
    }

    /// Access the signal value safely with a closure, without tracking dependencies.
    ///
    /// # Example
    ///
    /// ```
    /// # use flux_state::{Runtime, Signal};
    /// # let runtime = Runtime::new();
    /// let count = Signal::new(runtime, vec![1, 2, 3]);
    ///
    /// // Access without tracking
    /// let _len = count.with_untracked(|v| v.len());
    /// ```
    ///
    /// # Panics
    ///
    /// - Panics if the internal lock is poisoned.
    ///
    pub fn with_untracked<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let guard = self
            .handle
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        f(&*guard)
    }

    /// Get a reference to the runtime.
    pub fn runtime(&self) -> &Arc<Runtime> {
        &self.runtime
    }

    /// Set a debug label for this signal (only available with "nova" feature).
    #[cfg(feature = "nova")]
    pub fn with_label(self, label: impl Into<String>) -> Self {
        self.runtime.set_label(self.id, label.into());
        self
    }
}

impl<T: Clone + 'static + Send + Sync> ReadSignal<T> {
    /// Get the current value and track this dependency.
    ///
    /// Call this inside an Effect or Computed closure to subscribe to updates.
    ///
    /// If called outside of a tracking context (e.g., in `main` or a regular function),
    /// it will simply return the current value without creating a subscription.
    ///
    /// # Example
    ///
    /// ```
    /// use flux_state::{Runtime, Signal, Effect};
    ///
    /// let runtime = Runtime::new();
    /// let count = Signal::new(runtime.clone(), 0);
    /// let (read, _) = count.split();
    ///
    /// // 1. Inside an effect (Tracks dependency)
    /// let read_clone = read.clone();
    /// let _e = Effect::new(runtime, move || {
    ///     println!("Count is: {}", read_clone.get()); // Auto-subscribes
    /// });
    ///
    /// // 2. Outside an effect (Just reads value)
    /// let value = read.get(); // No tracking
    /// ```
    ///
    /// # Panics
    ///
    /// - Panics if the internal lock is poisoned.
    ///
    pub fn get(&self) -> T {
        self.with(|v| v.clone())
    }

    /// Get the current value without tracking dependencies.
    ///
    /// Use this when you want to read the value without causing the Effect
    /// to re-run when this signal changes.
    ///
    /// # Example
    ///
    /// ```
    /// use flux_state::{Runtime, Signal, Effect};
    ///
    /// let runtime = Runtime::new();
    /// let count = Signal::new(runtime.clone(), 0);
    /// let (read, _) = count.split();
    ///
    /// let _effect = Effect::new(runtime, move || {
    ///     // This effect will NOT re-run when count changes
    ///     println!("Count is: {}", read.get_untracked());
    /// });
    /// ```
    ///
    /// # Panics
    ///
    /// - Panics if the internal lock is poisoned.
    ///
    pub fn get_untracked(&self) -> T {
        self.with_untracked(|v| v.clone())
    }
}

impl<T: 'static + Send + Sync> ReadSignal<T> {
    /// Access the signal value safely with a closure.
    ///
    /// This method allows accessing the value without cloning it.
    /// It automatically tracks dependencies.
    ///
    /// # Example
    ///
    /// ```
    /// # use flux_state::{Runtime, Signal};
    /// # let runtime = Runtime::new();
    /// let count = Signal::new(runtime, vec![1, 2, 3]);
    /// let (read, _) = count.split();
    ///
    /// // Access length without cloning the vector
    /// let len = read.with(|v| v.len());
    /// assert_eq!(len, 3);
    /// ```
    ///
    /// # Panics
    ///
    /// - Panics if the internal lock is poisoned.
    ///
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        if self.is_computed {
            // Optimistically try to get fresh value with tracking in one go
            let handle = if let Some(h) = self.runtime.track_and_get_computed_if_fresh(self.id) {
                h
            } else {
                // Slow path: value is stale or uninitialized
                self.runtime.recompute(self.id);
                self.runtime.get_computed_handle(self.id)
            };

            let guard = handle
                .downcast_ref::<RwLock<T>>()
                .expect("Type mismatch")
                .read()
                .unwrap();
            f(&*guard)
        } else {
            self.runtime.track(self.id);
            let guard = self
                .handle
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            f(&*guard)
        }
    }

    /// Access the signal value safely with a closure, without tracking dependencies.
    ///
    /// # Example
    ///
    /// ```
    /// # use flux_state::{Runtime, Signal};
    /// # let runtime = Runtime::new();
    /// let count = Signal::new(runtime, vec![1, 2, 3]);
    /// let (read, _) = count.split();
    ///
    /// // Access without tracking
    /// let _len = read.with_untracked(|v| v.len());
    /// ```
    ///
    /// # Panics
    ///
    /// - Panics if the internal lock is poisoned.
    ///
    pub fn with_untracked<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        if self.is_computed {
            // Optimistically try to get fresh value without tracking
            let handle = if let Some(h) = self.runtime.get_computed_if_fresh(self.id) {
                h
            } else {
                // Slow path: value is stale or uninitialized
                self.runtime.recompute(self.id);
                self.runtime.get_computed_handle(self.id)
            };

            let guard = handle
                .downcast_ref::<RwLock<T>>()
                .expect("Type mismatch")
                .read()
                .unwrap();
            f(&*guard)
        } else {
            let guard = self
                .handle
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            f(&*guard)
        }
    }

    /// Get a reference to the runtime.
    pub fn runtime(&self) -> &Arc<Runtime> {
        &self.runtime
    }

    /// Internal: Create a ReadSignal from a computed node.
    pub(crate) fn from_computed(id: NodeId, runtime: Arc<Runtime>, handle: Arc<RwLock<T>>) -> Self {
        Self {
            id,
            runtime,
            handle,
            is_computed: true,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: 'static + Send + Sync> WriteSignal<T> {
    /// Set a new value and notify dependents.
    ///
    /// # Example
    ///
    /// ```
    /// # use flux_state::{Runtime, Signal, Effect};
    /// # let runtime = Runtime::new();
    /// let count = Signal::new(runtime.clone(), 0);
    /// let (read, write) = count.split();
    ///
    /// let read_clone = read.clone();
    /// let _e = Effect::new(runtime, move || {
    ///     println!("Value: {}", read_clone.get());
    /// });
    ///
    /// write.set(42); // Trigger effect update
    /// ```
    ///
    /// # Panics
    ///
    /// - Panics if the internal lock is poisoned.
    ///
    pub fn set(&self, value: T) {
        let old_value = {
            let mut guard = self
                .handle
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            std::mem::replace(&mut *guard, value)
        };
        drop(old_value);
        self.runtime.notify(self.id);
    }

    /// Update the value with a function and notify dependents.
    ///
    /// The closure receives a mutable reference `&mut T` to the current value.
    ///
    /// # Example
    ///
    /// ```
    /// # use flux_state::{Runtime, Signal, Effect};
    /// # let runtime = Runtime::new();
    /// let count = Signal::new(runtime.clone(), 0);
    /// let (read, write) = count.split();
    ///
    /// let read_clone = read.clone();
    /// let _e = Effect::new(runtime, move || {
    ///     println!("Value: {}", read_clone.get());
    /// });
    ///
    /// // Increment value in place
    /// write.update(|c| *c += 1); // Trigger effect update
    /// ```
    ///
    /// # Panics
    ///
    /// - Panics if the internal lock is poisoned.
    ///
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        {
            let mut guard = self
                .handle
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            f(&mut *guard);
        }
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
    fn test_update_triggers_effects_and_computed() {
        let runtime = Runtime::new();
        let signal = Signal::new(Arc::clone(&runtime), vec![1, 2, 3]);
        let (read, write) = signal.split();

        // Create a computed value derived from the signal
        let read_for_computed = read.clone();
        let computed = crate::Computed::new(Arc::clone(&runtime), move || {
            read_for_computed.with(|v| v.len())
        });

        // Create an effect that logs the signal's updates
        let log = Arc::new(std::sync::Mutex::new(Vec::new()));
        let log_clone = Arc::clone(&log);
        let read_for_effect = read.clone();
        let _effect = crate::Effect::new(Arc::clone(&runtime), move || {
            log_clone
                .lock()
                .unwrap()
                .push(read_for_effect.with(|v| v.clone()));
        });

        // Initial state
        assert_eq!(computed.get(), 3);
        assert_eq!(
            *log.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            vec![vec![1, 2, 3]]
        );

        // Mutate the signal using update()
        write.update(|v| {
            v.push(4);
            v.push(5);
        });

        // Assert that computed value is updated
        assert_eq!(computed.get(), 5);

        // Assert that effect ran again and logged the new state
        assert_eq!(
            *log.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            vec![vec![1, 2, 3], vec![1, 2, 3, 4, 5]]
        );
    }

    #[test]
    fn test_get_untracked() {
        let runtime = Runtime::new();
        let signal = Signal::new(Arc::clone(&runtime), 1);
        let (read, _write) = signal.split();
        assert_eq!(read.get_untracked(), 1);
    }

    #[test]
    fn test_with() {
        let runtime = Runtime::new();
        let signal = Signal::new(Arc::clone(&runtime), vec![1, 2, 3]);
        let (read, _) = signal.split();

        let len = read.with(|v| v.len());
        assert_eq!(len, 3);
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

    #[test]
    #[cfg(feature = "nova")]
    fn test_signal_with_label() {
        let runtime = Runtime::new();
        let signal_initial = Signal::new(runtime.clone(), 42);
        let id_before = signal_initial.id;

        let signal = signal_initial.with_label("my_signal");
        assert_eq!(signal.id, id_before);

        let (read, _write) = signal.clone().split();
        assert_eq!(read.get(), 42);

        // ensure arc pointers are exactly the same
        assert!(Arc::ptr_eq(&read.runtime, &runtime));

        let graph = runtime.inspect_graph();
        let node = graph.nodes.iter().find(|n| n.id == signal.id).unwrap();
        assert_eq!(node.label, "my_signal");
    }

    #[test]
    fn test_set_drop_deadlock_prevention() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime.clone(), None);
        let (_, write) = signal.split();

        #[derive(Clone)]
        struct DropTrigger {
            write: WriteSignal<Option<DropTrigger>>,
        }

        impl Drop for DropTrigger {
            fn drop(&mut self) {
                // If the value is dropped inside the write guard,
                // this `set` call will deadlock.
                self.write.set(None);
            }
        }

        // Set the value that will trigger the deadlock on drop
        write.set(Some(DropTrigger {
            write: write.clone(),
        }));

        // Replace the value to drop it
        write.set(None);
    }

    #[test]
    fn test_signal_debug_locked() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime.clone(), 42);
        let (read, write) = signal.clone().split();

        // Lock the signal manually
        let handle = runtime.get_signal_handle(signal.id);
        let guard = handle
            .downcast_ref::<std::sync::RwLock<i32>>()
            .unwrap()
            .write()
            .unwrap();

        let signal_debug = format!("{:?}", signal);
        assert!(signal_debug.contains("value: <locked>"));

        let read_debug = format!("{:?}", read);
        assert!(read_debug.contains("value: <locked>"));

        let write_debug = format!("{:?}", write);
        assert!(write_debug.contains("value: <locked>"));

        drop(guard);
    }
}
