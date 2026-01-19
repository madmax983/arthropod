//! Unit tests for PlatformError types.

use plat_core::PlatformError;

#[test]
fn test_platform_error_window_creation() {
    let error = PlatformError::WindowCreation("Invalid window style".to_string());
    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Failed to create window"));
    assert!(error_msg.contains("Invalid window style"));
}

#[test]
fn test_platform_error_initialization() {
    let error = PlatformError::Initialization("Failed to get module handle".to_string());
    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Platform initialization failed"));
    assert!(error_msg.contains("Failed to get module handle"));
}

#[test]
fn test_platform_error_event_loop() {
    let error = PlatformError::EventLoop("Message dispatch failed".to_string());
    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Event loop error"));
    assert!(error_msg.contains("Message dispatch failed"));
}

#[test]
fn test_platform_error_debug() {
    let error = PlatformError::WindowCreation("test".to_string());
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("WindowCreation"));
}

#[test]
fn test_platform_error_is_error() {
    // Test that PlatformError implements std::error::Error
    let error: Box<dyn std::error::Error> =
        Box::new(PlatformError::WindowCreation("test".to_string()));
    assert!(error.to_string().contains("Failed to create window"));
}
