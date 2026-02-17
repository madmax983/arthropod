fn main() {
    let mut opt = Some(1);
    let r = &mut opt;
    // This should fail if the reviewer is correct
    let _ = r.and_then(|x| Some(x + 1));
}
