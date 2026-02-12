use render_engine::backend::wgpu::context::{ProbeCaps, WebBackend, select_web_backend};

#[test]
fn test_backend_probe_prefers_webgpu_then_falls_back_to_gl() {
    let caps_webgpu = ProbeCaps {
        webgpu_available: true,
        webgl2_available: true,
    };
    assert_eq!(select_web_backend(caps_webgpu), WebBackend::WebGpu);

    let caps_gl = ProbeCaps {
        webgpu_available: false,
        webgl2_available: true,
    };
    assert_eq!(select_web_backend(caps_gl), WebBackend::WebGl2);
}
