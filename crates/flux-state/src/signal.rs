//! Signal primitive - the atomic unit of reactive state.

use crate::runtime::{NodeId, Runtime};
use std::sync::{Arc, Mutex};

/// A reactive signal - the atomic unit of state.
pub struct Signal<T> {
    id: NodeId,
    runtime: Arc<Runtime>,
    _marker: std::marker::PhantomData<T>,
}

/// Read-only view of a signal.
#[derive(Clone)]
pub struct ReadSignal<T> {
    id: NodeId,
    runtime: Arc<Runtime>,
    _marker: std::marker::PhantomData<T>,
}

/// Write-only view of a signal.
#[derive(Clone)]
pub struct WriteSignal<T> {
    id: NodeId,
    runtime: Arc<Runtime>,
    _marker: std::marker::PhantomData<T>,
}

impl<T: 'static + Send> Signal<T> {
    /// Create a new signal with an initial value.
    pub fn new(runtime: Arc<Runtime>, value: T) -> Self {
        let id = runtime.create_signal(Box::new(Mutex::new(value)));
        Self {
            id,
            runtime,
            _marker: std::marker::PhantomData,
        }
    }

    /// Split into read and write handles.
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
    /// Get the current value (tracks dependency).
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

    /// Get the current value without tracking.
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
    /// Set a new value.
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

    /// Update the value with a function.
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
