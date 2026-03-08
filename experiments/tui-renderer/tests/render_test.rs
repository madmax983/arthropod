use ratatui::backend::TestBackend;
use render_engine::{Color, Scene};
use tui_renderer::TuiBackend;

#[test]
fn test_backend_implements_trait() {
    // Use TestBackend for testing to avoid TTY requirement
    let backend = TestBackend::new(80, 24);
    let mut backend = TuiBackend::new_with_backend(backend).unwrap();
    let scene = Scene::new();

    // Set clear color
    backend.set_clear_color(Color::BLACK);

    // Resize
    backend.resize(80, 24);

    // Render
    let result = backend.render(&scene);
    assert!(result.is_ok());
}
