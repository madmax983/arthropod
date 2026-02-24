//! Effects - side effects that run when dependencies change.
//!
//! Effects are the "sinks" of the reactive graph. They are used to perform side effects
//! (like updating the DOM, logging, or making network requests) in response to state changes.

use crate::runtime::{NodeId, Runtime};
use std::sync::Arc;

/// An effect that runs automatically when its dependencies change.
///
/// Effects auto-track signals accessed within their closure. When any of those
/// signals change, the effect re-runs.
///
/// # Lifecycle
///
/// 1. **Creation**: The effect runs immediately to detect dependencies.
/// 2. **Update**: When a dependency changes, the effect is scheduled to run again.
/// 3. **Cleanup**: When the `Effect` struct is dropped, it unsubscribes from all signals.
///
/// # Example
///
/// ```
/// use flux_state::{Runtime, Signal, Effect};
///
/// let runtime = Runtime::new();
/// let count = Signal::new(runtime.clone(), 0);
/// let (read, write) = count.split();
///
/// // Create an effect that prints whenever count changes
/// let read_clone = read.clone();
/// let _effect = Effect::new(runtime.clone(), move || {
///     println!("Count changed to: {}", read_clone.get());
/// });
///
/// write.set(1); // Prints "Count changed to: 1"
/// ```
///
/// # Common Patterns
///
/// ## Resource Cleanup
///
/// If your effect creates resources (like timers, network connections, or DOM nodes),
/// you should rely on `Drop` implementations of the captured variables to clean them up.
/// The effect closure itself is re-run from scratch on every update, so any
/// local variables are dropped before the next run.
///
/// ```rust
/// # use flux_state::{Runtime, Effect};
/// # let runtime = Runtime::new();
/// struct MyResource;
/// impl Drop for MyResource {
///     fn drop(&mut self) { println!("Cleaning up!"); }
/// }
///
/// let _effect = Effect::new(runtime.clone(), || {
///     let _res = MyResource;
///     // ... do something with resource
///     // _res is dropped when the closure ends or before next run
/// });
/// ```
///
/// # Pitfalls
///
/// Effects are **dropped immediately** if they are not bound to a variable,
/// because `Effect` implements `Drop` to clean up dependencies.
///
/// ❌ **Wrong:**
/// ```rust
/// # use flux_state::{Runtime, Effect};
/// # let runtime = Runtime::new();
/// // This effect runs once, then is dropped and stopped immediately!
/// Effect::new(runtime.clone(), || println!("I will only run once!"));
/// ```
///
/// ✅ **Correct:**
/// ```rust
/// # use flux_state::{Runtime, Effect};
/// # let runtime = Runtime::new();
/// // Assign to `_variable` (not `_`) to keep it alive
/// let _keep_alive = Effect::new(runtime.clone(), || println!("I will keep running!"));
/// ```
#[must_use = "Effects are dropped (and stopped) immediately if not stored. Assign to a variable to keep alive."]
pub struct Effect {
    id: NodeId,
    runtime: Arc<Runtime>,
}

impl Effect {
    /// Create a new effect.
    ///
    /// The `effect_fn` will run immediately to establish initial dependencies.
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
    /// let _effect = Effect::new(runtime, move || {
    ///     println!("Count is: {}", read_clone.get());
    /// });
    ///
    /// write.set(1);
    /// ```
    ///
    /// # Warning
    ///
    /// The returned `Effect` handle **must be kept alive** (e.g., assigned to a variable).
    /// If the `Effect` struct is dropped, the effect is immediately stopped and unsubscribed
    /// from all dependencies.
    pub fn new<F>(runtime: Arc<Runtime>, effect_fn: F) -> Self
    where
        F: Fn() + 'static + Send + Sync,
    {
        let id = runtime.create_effect(Arc::new(effect_fn));

        // Run the effect immediately to establish dependencies.
        // We use a custom guard to ensure that if the initial run panics,
        // the effect is disposed (unsubscribed) immediately, preventing
        // it from becoming an "orphaned" zombie that runs again later.
        struct ConstructionGuard {
            runtime: Arc<Runtime>,
            id: NodeId,
            success: bool,
        }

        impl Drop for ConstructionGuard {
            fn drop(&mut self) {
                if !self.success {
                    self.runtime.dispose_effect(self.id);
                }
            }
        }

        let mut guard = ConstructionGuard {
            runtime: Arc::clone(&runtime),
            id,
            success: false,
        };

        runtime.run_effect(id);

        // If we get here, execution succeeded
        guard.success = true;

        Self { id, runtime }
    }

    /// Set a debug label for this effect (only available with "nova" feature).
    #[cfg(feature = "nova")]
    pub fn with_label(self, label: impl Into<String>) -> Self {
        self.runtime.set_label(self.id, label.into());
        self
    }
}

impl Drop for Effect {
    fn drop(&mut self) {
        self.runtime.dispose_effect(self.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::Runtime;
    use crate::signal::Signal;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_effect_basic() {
        let runtime = Runtime::new();
        let count = Signal::new(runtime.clone(), 0);
        let (read, write) = count.split();

        let log = Arc::new(Mutex::new(Vec::new()));
        let log_clone = log.clone();

        let _effect = Effect::new(runtime, move || {
            log_clone.lock().unwrap().push(read.get());
        });

        // Initial run
        assert_eq!(*log.lock().unwrap(), vec![0]);

        // Update
        write.set(1);
        assert_eq!(*log.lock().unwrap(), vec![0, 1]);
    }

    #[test]
    fn test_effect_cleanup() {
        let runtime = Runtime::new();
        let count = Signal::new(runtime.clone(), 0);
        let (read, write) = count.split();

        let log = Arc::new(Mutex::new(0));
        let log_clone = log.clone();

        {
            let _effect = Effect::new(runtime, move || {
                let _ = read.get();
                *log_clone.lock().unwrap() += 1;
            });
            // Run 1 (init)
            assert_eq!(*log.lock().unwrap(), 1);

            write.set(1);
            // Run 2
            assert_eq!(*log.lock().unwrap(), 2);
        } // Effect dropped here

        write.set(2);
        // Should NOT run
        assert_eq!(*log.lock().unwrap(), 2);
    }
}
