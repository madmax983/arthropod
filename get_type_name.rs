use std::any::Any;

fn check_downcast(a: &dyn Any) -> Option<String> {
    if let Some(v) = a.downcast_ref::<std::sync::Arc<std::sync::RwLock<i32>>>() {
        if let Ok(guard) = v.try_read() {
            return Some(format!("{:?}", *guard));
        } else {
            return Some("<locked>".to_string());
        }
    }
    None
}

fn main() {
    let x: std::sync::Arc<std::sync::RwLock<i32>> = std::sync::Arc::new(std::sync::RwLock::new(42));
    let y: std::sync::Arc<dyn Any + Send + Sync> = std::sync::Arc::new(x);

    let a: &dyn Any = y.as_ref();
    println!("downcast Arc<RwLock<i32>>: {:?}", check_downcast(a));

    let z: std::sync::Arc<dyn Any + Send + Sync> = std::sync::Arc::new(std::sync::RwLock::new(42i32));
    let b: &dyn Any = z.as_ref();

    println!("downcast RwLock<i32>: {:?}", b.downcast_ref::<std::sync::RwLock<i32>>().is_some());
}
