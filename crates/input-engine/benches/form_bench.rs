use criterion::{Criterion, black_box, criterion_group, criterion_main};
use flux_state::{Runtime, Signal};
use indexmap::IndexMap;
use input_engine::form::{FormState, revalidate_form};
use input_engine::text::TextInputState;
use input_engine::validation::ValidationState;
use render_engine::NodeId;
use std::collections::HashMap;
use std::sync::Arc;

fn create_text_state(initial: &str) -> TextInputState {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime, initial.to_string());
    let (read, write) = signal.split();
    TextInputState {
        read_signal: read,
        write_signal: write,
        cursor_position: initial.chars().count(),
        readonly: false,
        max_length: None,
    }
}

fn bench_revalidate_form(c: &mut Criterion) {
    let mut form_states = HashMap::new();
    let mut text_input_states = IndexMap::new();
    let mut validators = HashMap::new();

    let form_id = NodeId(10);
    let mut field_mapping = IndexMap::new();

    for i in 0..100 {
        let field_id = NodeId(100 + i);
        field_mapping.insert(format!("field_{}", i), field_id);

        text_input_states.insert(field_id, create_text_state("test"));

        validators.insert(
            field_id,
            ValidationState {
                validator: Arc::new(|val| {
                    if val.len() > 2 {
                        Ok(())
                    } else {
                        Err("Too short".to_string())
                    }
                }),
                error: None,
            },
        );
    }

    form_states.insert(
        form_id,
        FormState {
            field_mapping,
            is_valid: true,
            on_submit: None,
            submit_error: None,
        },
    );

    c.bench_function("revalidate_form_100_fields", |b| {
        b.iter(|| {
            revalidate_form(
                black_box(form_id),
                black_box(&mut form_states),
                black_box(&text_input_states),
                black_box(&mut validators),
            );
        })
    });
}

criterion_group!(benches, bench_revalidate_form);
criterion_main!(benches);
