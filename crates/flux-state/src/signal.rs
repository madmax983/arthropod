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

// NOTE: We rely on auto-traits for Sync.
// Signal<T> is Sync if T is Sync (because RwLock<T> is Sync if T is Sync).
// This prevents sharing !Sync types (like RefCell) across threads, which avoids data races.

impl<T: std::fmt::Debug + 'static + Send + Sync> std::fmt::Debug for Signal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Track the dependency so that effects re-run when the signal changes.
        self.runtime.track(self.id);

        let handle = self.runtime.get_signal_handle(self.id);

        let guard = match handle.downcast_ref::<RwLock<T>>() {
            Some(m) => m.try_read(),
            None => {
                return write!(f, "Signal(id: {:?}, value: <type mismatch>)", self.id);
            }
        };

        match guard {
            Ok(val) => write!(f, "Signal(id: {:?}, value: {:?})", self.id, *val),
            Err(_) => write!(f, "Signal(id: {:?}, value: <locked>)", self.id),
        }
    }
}

impl<T: std::fmt::Debug + 'static + Send + Sync> std::fmt::Debug for ReadSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Track the dependency so that effects re-run when the signal changes.
        self.runtime.track(self.id);

        let handle = self.runtime.get_signal_handle(self.id);

        let guard = match handle.downcast_ref::<RwLock<T>>() {
            Some(m) => m.try_read(),
            None => {
                return write!(f, "ReadSignal(id: {:?}, value: <type mismatch>)", self.id);
            }
        };

        match guard {
            Ok(val) => write!(f, "ReadSignal(id: {:?}, value: {:?})", self.id, *val),
            Err(_) => write!(f, "ReadSignal(id: {:?}, value: <locked>)", self.id),
        }
    }
}

impl<T: std::fmt::Debug + 'static + Send + Sync> std::fmt::Debug for WriteSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Track the dependency so that effects re-run when the signal changes.
        self.runtime.track(self.id);

        let handle = self.runtime.get_signal_handle(self.id);

        let guard = match handle.downcast_ref::<RwLock<T>>() {
            Some(m) => m.try_read(),
            None => {
                return write!(f, "WriteSignal(id: {:?}, value: <type mismatch>)", self.id);
            }
        };

        match guard {
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
        let id = runtime.create_signal(Arc::new(RwLock::new(value)));
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
                _marker: std::marker::PhantomData,
            },
            WriteSignal {
                id: self.id,
                runtime: Arc::clone(&self.runtime),
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
    /// - Panics if the stored type does not match `T`.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.runtime.track(self.id);
        let handle = self.runtime.get_signal_handle(self.id);
        let guard = handle
            .downcast_ref::<RwLock<T>>()
            .expect("Type mismatch")
            .read()
            .unwrap();
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
    /// - Panics if the stored type does not match `T`.
    pub fn with_untracked<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let handle = self.runtime.get_signal_handle(self.id);
        let guard = handle
            .downcast_ref::<RwLock<T>>()
            .expect("Type mismatch")
            .read()
            .unwrap();
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
    /// - Panics if the stored type does not match `T`.
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
    /// - Panics if the stored type does not match `T`.
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
    /// - Panics if the stored type does not match `T`.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.runtime.track(self.id);
        let handle = self.runtime.get_signal_handle(self.id);
        let guard = handle
            .downcast_ref::<RwLock<T>>()
            .expect("Type mismatch")
            .read()
            .unwrap();
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
    /// let (read, _) = count.split();
    ///
    /// // Access without tracking
    /// let _len = read.with_untracked(|v| v.len());
    /// ```
    ///
    /// # Panics
    ///
    /// - Panics if the internal lock is poisoned.
    /// - Panics if the stored type does not match `T`.
    pub fn with_untracked<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let handle = self.runtime.get_signal_handle(self.id);
        let guard = handle
            .downcast_ref::<RwLock<T>>()
            .expect("Type mismatch")
            .read()
            .unwrap();
        f(&*guard)
    }

    /// Get a reference to the runtime.
    pub fn runtime(&self) -> &Arc<Runtime> {
        &self.runtime
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
    /// - Panics if the stored type does not match `T`.
    pub fn set(&self, value: T) {
        let handle = self.runtime.get_signal_handle(self.id);
        {
            let mut guard = handle
                .downcast_ref::<RwLock<T>>()
                .expect("Type mismatch")
                .write()
                .unwrap();
            *guard = value;
        }
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
    /// - Panics if the stored type does not match `T`.
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        let handle = self.runtime.get_signal_handle(self.id);
        {
            let mut guard = handle
                .downcast_ref::<RwLock<T>>()
                .expect("Type mismatch")
                .write()
                .unwrap();
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
}
