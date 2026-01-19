//! Signal registry for remote control of reactive state.
//!
//! Allows registering named signals that can be controlled via MCP tools.

use anyhow::{Result, anyhow};
use flux_state::{ReadSignal, WriteSignal};
use hashbrown::HashMap;
use render_engine::Color;

/// Type-erased signal entry
pub enum SignalEntry {
    /// Color signal (RGBA)
    ColorSignal {
        read: ReadSignal<Color>,
        write: WriteSignal<Color>,
    },

    /// F32 signal
    F32Signal {
        read: ReadSignal<f32>,
        write: WriteSignal<f32>,
    },

    /// Bool signal
    BoolSignal {
        read: ReadSignal<bool>,
        write: WriteSignal<bool>,
    },
    // More types can be added as needed
}

/// Registry for named signals that can be controlled via MCP
pub struct SignalRegistry {
    entries: HashMap<String, SignalEntry>,
}

impl SignalRegistry {
    /// Create a new empty signal registry
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Register a color signal
    pub fn register_color(
        &mut self,
        name: String,
        read: ReadSignal<Color>,
        write: WriteSignal<Color>,
    ) {
        self.entries
            .insert(name, SignalEntry::ColorSignal { read, write });
    }

    /// Register an f32 signal
    pub fn register_f32(&mut self, name: String, read: ReadSignal<f32>, write: WriteSignal<f32>) {
        self.entries
            .insert(name, SignalEntry::F32Signal { read, write });
    }

    /// Register a bool signal
    pub fn register_bool(
        &mut self,
        name: String,
        read: ReadSignal<bool>,
        write: WriteSignal<bool>,
    ) {
        self.entries
            .insert(name, SignalEntry::BoolSignal { read, write });
    }

    /// Set a color signal value
    pub fn set_color(&mut self, name: &str, value: Color) -> Result<Color> {
        match self.entries.get(name) {
            Some(SignalEntry::ColorSignal { read, write }) => {
                let old = read.get_untracked();
                write.set(value);
                Ok(old)
            }
            Some(_) => Err(anyhow!("Signal '{}' is not a Color signal", name)),
            None => Err(anyhow!("Signal '{}' not found", name)),
        }
    }

    /// Set an f32 signal value
    pub fn set_f32(&mut self, name: &str, value: f32) -> Result<f32> {
        match self.entries.get(name) {
            Some(SignalEntry::F32Signal { read, write }) => {
                let old = read.get_untracked();
                write.set(value);
                Ok(old)
            }
            Some(_) => Err(anyhow!("Signal '{}' is not an f32 signal", name)),
            None => Err(anyhow!("Signal '{}' not found", name)),
        }
    }

    /// Set a bool signal value
    pub fn set_bool(&mut self, name: &str, value: bool) -> Result<bool> {
        match self.entries.get(name) {
            Some(SignalEntry::BoolSignal { read, write }) => {
                let old = read.get_untracked();
                write.set(value);
                Ok(old)
            }
            Some(_) => Err(anyhow!("Signal '{}' is not a bool signal", name)),
            None => Err(anyhow!("Signal '{}' not found", name)),
        }
    }

    /// Get a color signal value (non-reactive)
    pub fn get_color(&self, name: &str) -> Result<Color> {
        match self.entries.get(name) {
            Some(SignalEntry::ColorSignal { read, .. }) => Ok(read.get_untracked()),
            Some(_) => Err(anyhow!("Signal '{}' is not a Color signal", name)),
            None => Err(anyhow!("Signal '{}' not found", name)),
        }
    }

    /// Get an f32 signal value (non-reactive)
    pub fn get_f32(&self, name: &str) -> Result<f32> {
        match self.entries.get(name) {
            Some(SignalEntry::F32Signal { read, .. }) => Ok(read.get_untracked()),
            Some(_) => Err(anyhow!("Signal '{}' is not an f32 signal", name)),
            None => Err(anyhow!("Signal '{}' not found", name)),
        }
    }

    /// Get a bool signal value (non-reactive)
    pub fn get_bool(&self, name: &str) -> Result<bool> {
        match self.entries.get(name) {
            Some(SignalEntry::BoolSignal { read, .. }) => Ok(read.get_untracked()),
            Some(_) => Err(anyhow!("Signal '{}' is not a bool signal", name)),
            None => Err(anyhow!("Signal '{}' not found", name)),
        }
    }

    /// Check if a signal exists
    pub fn contains(&self, name: &str) -> bool {
        self.entries.contains_key(name)
    }

    /// List all registered signal names
    pub fn list_signals(&self) -> Vec<String> {
        self.entries.keys().cloned().collect()
    }

    /// Get signal type as string
    pub fn get_type(&self, name: &str) -> Option<&'static str> {
        self.entries.get(name).map(|entry| match entry {
            SignalEntry::ColorSignal { .. } => "color",
            SignalEntry::F32Signal { .. } => "f32",
            SignalEntry::BoolSignal { .. } => "bool",
        })
    }
}

impl Default for SignalRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_state::{Runtime, Signal};

    #[test]
    fn test_register_and_set_color_signal() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime.clone(), Color::RED);
        let (read, write) = signal.split();

        let mut registry = SignalRegistry::new();
        registry.register_color("test_color".to_string(), read.clone(), write);

        // Set new color
        let old = registry.set_color("test_color", Color::BLUE).unwrap();
        assert_eq!(old.to_array(), Color::RED.to_array());

        // Verify signal was updated
        assert_eq!(read.get_untracked().to_array(), Color::BLUE.to_array());
    }

    #[test]
    fn test_register_and_set_f32_signal() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime.clone(), 1.0);
        let (read, write) = signal.split();

        let mut registry = SignalRegistry::new();
        registry.register_f32("test_f32".to_string(), read.clone(), write);

        let old = registry.set_f32("test_f32", 2.5).unwrap();
        assert_eq!(old, 1.0);
        assert_eq!(read.get_untracked(), 2.5);
    }

    #[test]
    fn test_type_mismatch_error() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime.clone(), Color::RED);
        let (read, write) = signal.split();

        let mut registry = SignalRegistry::new();
        registry.register_color("test_color".to_string(), read, write);

        // Try to set as f32 (should fail)
        let result = registry.set_f32("test_color", 1.0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not an f32"));
    }

    #[test]
    fn test_signal_not_found() {
        let mut registry = SignalRegistry::new();

        let result = registry.set_color("nonexistent", Color::RED);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_list_signals() {
        let runtime = Runtime::new();
        let mut registry = SignalRegistry::new();

        let color_sig = Signal::new(runtime.clone(), Color::RED);
        let f32_sig = Signal::new(runtime.clone(), 1.0);

        let (r1, w1) = color_sig.split();
        let (r2, w2) = f32_sig.split();

        registry.register_color("color1".to_string(), r1, w1);
        registry.register_f32("opacity".to_string(), r2, w2);

        let signals = registry.list_signals();
        assert_eq!(signals.len(), 2);
        assert!(signals.contains(&"color1".to_string()));
        assert!(signals.contains(&"opacity".to_string()));
    }

    #[test]
    fn test_get_signal_type() {
        let runtime = Runtime::new();
        let mut registry = SignalRegistry::new();

        let color_sig = Signal::new(runtime.clone(), Color::RED);
        let (read, write) = color_sig.split();
        registry.register_color("test".to_string(), read, write);

        assert_eq!(registry.get_type("test"), Some("color"));
        assert_eq!(registry.get_type("nonexistent"), None);
    }
}
