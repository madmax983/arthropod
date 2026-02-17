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

// SAFETY: Signal uses internal Mutex locking via Runtime, so it is safe to share
// between threads even if T is !Sync (e.g., RefCell), as long as T is Send.
// T must be Send because it is stored in Arc<dyn Any + Send + Sync>.
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
    ///
    /// # Panics
    ///
    /// Panics if the runtime fails to allocate a new signal ID (unlikely).
    pub fn new(runtime: Arc<Runtime>, value: T) -> Self {
        let id = runtime.create_signal(Arc::new(Mutex::new(value)));
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
    /// - Panics if the internal mutex is poisoned.
    /// - Panics if the stored type does not match `T`.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.runtime.track(self.id);
        let handle = self.runtime.get_signal_handle(self.id);
        let guard = handle
            .downcast_ref::<Mutex<T>>()
            .expect("Type mismatch")
            .lock()
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
    /// - Panics if the internal mutex is poisoned.
    /// - Panics if the stored type does not match `T`.
    pub fn with_untracked<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let handle = self.runtime.get_signal_handle(self.id);
        let guard = handle
            .downcast_ref::<Mutex<T>>()
            .expect("Type mismatch")
            .lock()
            .unwrap();
        f(&*guard)
    }

    /// Get a reference to the runtime.
    pub fn runtime(&self) -> &Arc<Runtime> {
        &self.runtime
    }
}

impl<T: Clone + 'static + Send> ReadSignal<T> {
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
    /// - Panics if the internal mutex is poisoned.
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
    /// Effect::new(runtime, move || {
    ///     // This effect will NOT re-run when count changes
    ///     println!("Count is: {}", read.get_untracked());
    /// });
    /// ```
    ///
    /// # Panics
    ///
    /// - Panics if the internal mutex is poisoned.
    /// - Panics if the stored type does not match `T`.
    pub fn get_untracked(&self) -> T {
        self.with_untracked(|v| v.clone())
    }
}

impl<T: 'static + Send> ReadSignal<T> {
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
    /// - Panics if the internal mutex is poisoned.
    /// - Panics if the stored type does not match `T`.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.runtime.track(self.id);
        let handle = self.runtime.get_signal_handle(self.id);
        let guard = handle
            .downcast_ref::<Mutex<T>>()
            .expect("Type mismatch")
            .lock()
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
    /// - Panics if the internal mutex is poisoned.
    /// - Panics if the stored type does not match `T`.
    pub fn with_untracked<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let handle = self.runtime.get_signal_handle(self.id);
        let guard = handle
            .downcast_ref::<Mutex<T>>()
            .expect("Type mismatch")
            .lock()
            .unwrap();
        f(&*guard)
    }

    /// Get a reference to the runtime.
    pub fn runtime(&self) -> &Arc<Runtime> {
        &self.runtime
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
    ///
    /// # Panics
    ///
    /// - Panics if the internal mutex is poisoned.
    /// - Panics if the stored type does not match `T`.
    pub fn set(&self, value: T) {
        let handle = self.runtime.get_signal_handle(self.id);
        {
            let mut guard = handle
                .downcast_ref::<Mutex<T>>()
                .expect("Type mismatch")
                .lock()
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
    /// # use flux_state::{Runtime, Signal};
    /// # let runtime = Runtime::new();
    /// let count = Signal::new(runtime, 0);
    /// let (_, write) = count.split();
    ///
    /// // The closure argument `c` is `&mut i32`
    /// write.update(|c| *c += 1);
    /// ```
    ///
    /// # Panics
    ///
    /// - Panics if the internal mutex is poisoned.
    /// - Panics if the stored type does not match `T`.
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        let handle = self.runtime.get_signal_handle(self.id);
        {
            let mut guard = handle
                .downcast_ref::<Mutex<T>>()
                .expect("Type mismatch")
                .lock()
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
