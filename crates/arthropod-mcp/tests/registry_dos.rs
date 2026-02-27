use arthropod_mcp::registry::SignalRegistry;
use flux_state::{Runtime, Signal};
use render_engine::Color;

#[test]
fn test_signal_limit_enforcement() {
    let runtime = Runtime::new();
    let mut registry = SignalRegistry::new();

    // Register 1000 signals (allowed)
    for i in 0..1000 {
        let signal = Signal::new(runtime.clone(), Color::RED);
        let (read, write) = signal.split();
        registry
            .register_color(format!("sig_{}", i), read, write)
            .unwrap();
    }

    // Attempt to register 1001st signal (should fail)
    let signal = Signal::new(runtime.clone(), Color::RED);
    let (read, write) = signal.split();

    let result = registry.register_color("sig_overflow".to_string(), read, write);
    assert!(result.is_err(), "Registry allowed more than 1000 signals!");
    assert!(result.unwrap_err().to_string().contains("Registry full"));
}

#[test]
fn test_signal_name_length_limit() {
    let runtime = Runtime::new();
    let mut registry = SignalRegistry::new();
    let signal = Signal::new(runtime.clone(), Color::RED);
    let (read, write) = signal.split();

    let long_name = "a".repeat(65);
    let result = registry.register_color(long_name.clone(), read, write);

    assert!(result.is_err(), "Registry allowed name > 64 chars!");
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Signal name too long")
    );
}
