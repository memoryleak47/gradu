use std::sync::atomic::{AtomicUsize, Ordering};

pub fn fresh() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    COUNTER.fetch_add(1, Ordering::SeqCst)
}
