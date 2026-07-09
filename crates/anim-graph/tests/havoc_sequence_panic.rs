use anim_graph::Evaluable;
use anim_graph::sequence::Sequence;
use anim_graph::stagger::Stagger;

#[test]
fn test_havoc_sequence_and_stagger_panic() {
    let empty_seq: Sequence<f32> = Sequence::new(vec![]);
    let _ = empty_seq.evaluate(0.5);

    let empty_stag: Stagger<f32> = Stagger::new(vec![], 0.5);
    let _ = empty_stag.evaluate_at(0, 0.5);
    let _ = empty_stag.evaluate(0.5);
}
