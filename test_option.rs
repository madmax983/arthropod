fn main() {
    let mut opt = Some(1);
    let r = &mut opt;
    // Fix: Use take() to move out value, transform, and replace
    *r = r.take().and_then(|x| Some(x + 1));

    // Verify the change occurred
    assert_eq!(opt, Some(2));
}
