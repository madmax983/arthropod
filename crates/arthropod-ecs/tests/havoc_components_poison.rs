use arthropod_ecs::components::OnA11yClick;
use std::sync::{Arc, Mutex};

#[test]
fn test_havoc_components_poison() {
    let clicked = Arc::new(Mutex::new(false));
    let clicked_clone = clicked.clone();

    // Create a poisoned lock
    let handle = std::thread::spawn(move || {
        let _guard = clicked_clone.lock().unwrap();
        panic!("Die!");
    });
    let _ = handle.join();

    // Now attempt to use the OnA11yClick struct that uses the poisoned lock
    let on_click = OnA11yClick::new(move || {
        let mut guard = clicked
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *guard = true;
    });

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        (on_click.callback)();
    }));

    assert!(result.is_ok(), "Component poisoned lock");
}
