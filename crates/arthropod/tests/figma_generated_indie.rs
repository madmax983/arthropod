use arthropod::figma_runtime::FigmaRuntime;

#[allow(dead_code)]
#[path = "../../../examples/generated/indie_make_generated_module.rs"]
mod indie_make_generated_module;

#[test]
fn indie_generated_runtime_emits_visible_render_instances() {
    let mut runtime: FigmaRuntime = indie_make_generated_module::indie_make_generated::runtime()
        .expect("generated indie runtime should initialize");
    runtime.apply_layout(1280.0, 720.0);

    let instances = runtime.collect_render_instances();
    assert!(
        !instances.is_empty(),
        "generated indie runtime should emit render instances (blank window regression)"
    );
    assert!(
        instances.iter().any(|instance| instance.color[3] > 0.0),
        "generated indie runtime should emit at least one visible instance"
    );
}
