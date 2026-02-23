//! Computed values - derived reactive state.
//!
//! Computed values are signals that are derived from other signals. They automatically
//! update when their dependencies change.
//!
//! # Lazy vs Eager Behavior
//!
//! - **Eager Initialization**: When you create a `Computed` value using [`Computed::new`],
//!   the closure is executed *immediately* to calculate the initial value. This ensures
//!   the value is always valid from the start.
//! - **Lazy Updates**: After initialization, the value is only re-calculated when:
//!   1. A dependency changes, AND
//!   2. The value is read (e.g., via `.get()`, `.with()`, or inside another Effect/Computed).
//!
//! If a dependency changes but the computed value is never read, the computation closure
//! will NOT run. This "pull-based" reactivity saves resources.

use crate::runtime::{NodeId, Runtime};
use std::sync::{Arc, Mutex};

/// A derived/computed value that automatically tracks dependencies.
///
/// Computed values are useful for deriving state from other signals without manually
/// syncing them. They memoize their result and only recompute when dependencies change.
///
/// # Example
///
/// ```
/// use flux_state::{Runtime, Signal, Computed};
///
/// let runtime = Runtime::new();
/// let count = Signal::new(runtime.clone(), 1);
/// let (read_count, _) = count.split();
///
/// // Create a computed value that doubles the count
/// // NOTE: This executes immediately to calculate the initial value (2)
/// let read_count_clone = read_count.clone();
/// let double_count = Computed::new(runtime.clone(), move || {
///     read_count_clone.get() * 2
/// });
///
/// assert_eq!(double_count.get(), 2);
/// ```
///
/// # The Diamond Problem
///
/// `Computed` correctly handles the "Diamond Problem" (shared dependencies), ensuring
/// that the computed value updates only once even if multiple paths lead back to the
/// same source signal.
///
/// ```text
///      A
///     / \
///    B   C
///     \ /
///      D
/// ```
///
/// In this graph:
/// 1. `A` updates.
/// 2. `B` and `C` are marked stale.
/// 3. `D` is marked stale.
/// 4. When `D` is read, it re-evaluates `B` and `C` (if needed), ensuring consistency without glitching.
///
/// ```
/// use flux_state::{Runtime, Signal, Computed};
///
/// let runtime = Runtime::new();
/// let a = Signal::new(runtime.clone(), 1);
/// let (r_a, w_a) = a.split();
///
/// // B depends on A
/// let r_a_1 = r_a.clone();
/// let b = Computed::new(runtime.clone(), move || r_a_1.get() * 2);
///
/// // C depends on A
/// let r_a_2 = r_a.clone();
/// let c = Computed::new(runtime.clone(), move || r_a_2.get() + 1);
///
/// // D depends on B and C
/// let b_c = b.clone();
/// let c_c = c.clone();
/// let d = Computed::new(runtime.clone(), move || b_c.get() + c_c.get());
///
/// // Initial: a=1 => b=2, c=2 => d=4
/// assert_eq!(d.get(), 4);
///
/// // Update A: a=2 => b=4, c=3 => d=7
/// w_a.set(2);
/// assert_eq!(d.get(), 7);
/// ```
#[derive(Clone)]
pub struct Computed<T> {
    id: NodeId,
    runtime: Arc<Runtime>,
    _marker: std::marker::PhantomData<T>,
}

impl<T: std::fmt::Debug + 'static + Send> std::fmt::Debug for Computed<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Track the dependency so that effects re-run when the computed value changes.
        self.runtime.track(self.id);

        // Recompute if stale to show fresh value.
        // We do this because debugging usually implies wanting to know the *current* state.
        if self.runtime.is_stale(self.id) {
            self.runtime.recompute(self.id);
        }

        let handle = self.runtime.get_computed_handle(self.id);

        let guard = match handle.downcast_ref::<Mutex<T>>() {
            Some(m) => m.try_lock(),
            None => {
                return write!(f, "Computed(id: {:?}, value: <type mismatch>)", self.id);
            }
        };

        match guard {
            Ok(val) => write!(f, "Computed(id: {:?}, value: {:?})", self.id, *val),
            Err(_) => write!(f, "Computed(id: {:?}, value: <locked>)", self.id),
        }
    }
}

impl<T: 'static + Send> Computed<T> {
    /// Create a new computed value.
    ///
    /// The `compute` closure will be called *immediately* to calculate the initial value,
    /// and then automatically called again whenever any signal it reads changes and the
    /// computed value is accessed.
    ///
    /// # Panics
    ///
    /// - Panics if the `compute` closure panics.
    /// - Panics if the runtime recursion limit (100) is exceeded (e.g., infinite dependency loop).
    pub fn new<F>(runtime: Arc<Runtime>, compute: F) -> Self
    where
        F: Fn() -> T + 'static + Send + Sync,
    {
        let id = runtime.create_computed(Arc::new(move || {
            Arc::new(Mutex::new(compute())) as Arc<dyn std::any::Any + Send + Sync>
        }));

        // Initialize the value by computing it once
        let result = Self {
            id,
            runtime: Arc::clone(&runtime),
            _marker: std::marker::PhantomData,
        };

        // Compute initial value
        result.runtime.recompute(result.id);

        result
    }

    /// Access the computed value safely with a closure.
    ///
    /// This method allows accessing the value without cloning it.
    /// It automatically tracks dependencies and recomputes if stale.
    ///
    /// # Example
    ///
    /// ```
    /// # use flux_state::{Runtime, Signal, Computed};
    /// # let runtime = Runtime::new();
    /// # let count = Signal::new(runtime.clone(), vec![1, 2, 3]);
    /// # let (read, _) = count.split();
    /// let computed = Computed::new(runtime.clone(), move || read.get());
    ///
    /// // Access the internal Vec without cloning it
    /// let len = computed.with(|v| v.len());
    /// assert_eq!(len, 3);
    /// ```
    ///
    /// # Panics
    ///
    /// - Panics if the internal mutex is poisoned.
    /// - Panics if the stored type does not match `T` (should not happen in safe code).
    /// - Panics if re-computation fails (e.g., dependency panic).
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.runtime.track(self.id);

        // Check if value is stale and recompute if needed
        if self.runtime.is_stale(self.id) {
            self.runtime.recompute(self.id);
        }

        let handle = self.runtime.get_computed_handle(self.id);
        let guard = handle
            .downcast_ref::<Mutex<T>>()
            .expect("Type mismatch")
            .lock()
            .unwrap();
        f(&*guard)
    }

    /// Access the computed value safely with a closure, without tracking dependencies.
    ///
    /// Note: This will still trigger a recompute if the value is stale, but will not
    /// subscribe the current context to this computed value.
    ///
    /// # Example
    ///
    /// ```
    /// # use flux_state::{Runtime, Signal, Computed};
    /// # let runtime = Runtime::new();
    /// # let count = Signal::new(runtime.clone(), 10);
    /// # let (read, _) = count.split();
    /// let computed = Computed::new(runtime.clone(), move || read.get() * 2);
    ///
    /// // Read value without subscribing the current effect/computed to updates
    /// let val = computed.with_untracked(|v| *v);
    /// assert_eq!(val, 20);
    /// ```
    ///
    /// # Panics
    ///
    /// - Panics if the internal mutex is poisoned.
    /// - Panics if the stored type does not match `T`.
    pub fn with_untracked<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        // Check if value is stale and recompute if needed
        if self.runtime.is_stale(self.id) {
            self.runtime.recompute(self.id);
        }

        let handle = self.runtime.get_computed_handle(self.id);
        let guard = handle
            .downcast_ref::<Mutex<T>>()
            .expect("Type mismatch")
            .lock()
            .unwrap();
        f(&*guard)
    }

    /// Set a debug label for this computed value (only available with "nova" feature).
    #[cfg(feature = "nova")]
    pub fn with_label(self, label: impl Into<String>) -> Self {
        self.runtime.set_label(self.id, label.into());
        self
    }
}

impl<T: Clone + 'static + Send> Computed<T> {
    /// Get the current value.
    ///
    /// This method:
    /// 1. Tracks the dependency if called inside an Effect or another Computed.
    /// 2. Recomputes the value if any dependencies have changed (lazy evaluation).
    /// 3. Returns the memoized value otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// # use flux_state::{Runtime, Signal, Computed};
    /// # let runtime = Runtime::new();
    /// # let count = Signal::new(runtime.clone(), 5);
    /// # let (read, _) = count.split();
    /// let computed = Computed::new(runtime.clone(), move || read.get() + 1);
    ///
    /// assert_eq!(computed.get(), 6);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the re-computation fails or internal state is corrupted.
    pub fn get(&self) -> T {
        self.with(|v| v.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::Runtime;
    use crate::signal::Signal;

    #[test]
    fn test_computed_basic() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime.clone(), 1);
        let (read, write) = signal.split();

        let read_clone = read.clone();
        let computed = Computed::new(runtime, move || read_clone.get() * 2);

        assert_eq!(computed.get(), 2);

        write.set(10);
        assert_eq!(computed.get(), 20);
    }

    #[test]
    fn test_diamond_pull() {
        let runtime = Runtime::new();
        let a = Signal::new(runtime.clone(), 1);
        let (r_a, w_a) = a.split();

        let r_a_1 = r_a.clone();
        let b = Computed::new(runtime.clone(), move || r_a_1.get() * 2);

        let r_a_2 = r_a.clone();
        let c = Computed::new(runtime.clone(), move || r_a_2.get() + 1);

        // D = B + C
        let b_c = b.clone();
        let c_c = c.clone();
        let d = Computed::new(runtime, move || b_c.get() + c_c.get());

        // Initial: a=1, b=2, c=2, d=4
        assert_eq!(d.get(), 4);

        w_a.set(2);
        // a=2, b=4, c=3, d=7
        assert_eq!(d.get(), 7);
    }

    #[test]
    fn test_dynamic_deps() {
        let runtime = Runtime::new();
        let toggle = Signal::new(runtime.clone(), true);
        let (r_toggle, w_toggle) = toggle.split();

        let a = Signal::new(runtime.clone(), 10);
        let (r_a, _w_a) = a.split();

        let b = Signal::new(runtime.clone(), 20);
        let (r_b, w_b) = b.split();

        let r_toggle_c = r_toggle.clone();
        let r_a_c = r_a.clone();
        let r_b_c = r_b.clone();

        let computed = Computed::new(runtime.clone(), move || {
            if r_toggle_c.get() {
                r_a_c.get()
            } else {
                r_b_c.get()
            }
        });

        assert_eq!(computed.get(), 10);

        // Update B. A is still active. Computed should not change.
        w_b.set(30);
        assert_eq!(computed.get(), 10);

        // Switch to B.
        w_toggle.set(false);
        assert_eq!(computed.get(), 30);
    }
}
