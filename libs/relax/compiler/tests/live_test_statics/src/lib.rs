use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

#[test]
fn affected() {
    assert_eq!(COUNTER.swap(1, Relaxed), identity_macro::identity!(0));
}

#[test]
fn unrelated() {
    #[cfg(feature = "park-worker")]
    {
        let (ready, started) = std::sync::mpsc::channel();
        std::thread::Builder::new().name("parked-test-worker".into()).spawn(move || {
            ready.send(()).unwrap();
            loop { std::thread::park(); }
        }).unwrap();
        started.recv().unwrap();
    }
}
