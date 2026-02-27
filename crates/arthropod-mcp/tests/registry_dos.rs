use arthropod_mcp::registry::SignalRegistry;
use flux_state::{Runtime, Signal};
use render_engine::Color;

#[test]
fn test_signal_limit() {
    let mut registry = SignalRegistry::new();
    let runtime = Runtime::new();
    let signal = Signal::new(runtime.clone(), Color::RED);

    // Register 1000 signals (allowed)
    for i in 0..1000 {
        let (read, write) = signal.clone().split();
        registry
            .register_color(format!("sig_{}", i), read, write)
            .expect("Should be able to register 1000 signals");
    }

    // Register 1001st signal (should fail)
    let (read, write) = signal.clone().split();
    let result = registry.register_color("sig_1001".to_string(), read, write);

    assert!(
        result.is_err(),
        "Should not be able to register more than 1000 signals"
    );
}

#[test]
fn test_name_length_limit() {
    let mut registry = SignalRegistry::new();
    let runtime = Runtime::new();
    let signal = Signal::new(runtime.clone(), Color::RED);
    let (read, write) = signal.split();

    // 64 chars (allowed)
    let name_64 = "a".repeat(64);
    registry
        .register_color(name_64, read.clone(), write.clone())
        .expect("Should accept 64-char name");

    // 65 chars (should fail)
    let name_65 = "a".repeat(65);
    let result = registry.register_color(name_65, read, write);

    assert!(
        result.is_err(),
        "Should not accept name longer than 64 chars"
    );
}
