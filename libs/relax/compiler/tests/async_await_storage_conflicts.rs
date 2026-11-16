// A Ready-path output copy must not overlap a saved awaitee whose destructor can
// still write its payload, even when the slots belong to different suspension states.
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

fn finish<F: Future>(f: F) -> F::Output {
    let mut f = Box::pin(f);
    let mut cx = Context::from_waker(Waker::noop());
    loop { if let Poll::Ready(value) = f.as_mut().poll(&mut cx) { return value; } }
}
struct Produced { pending: bool, drops: Arc<AtomicUsize>, payload: [u64; 4] }
impl Future for Produced {
    type Output = [u64; 4];
    fn poll(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.pending { self.pending = true; return Poll::Pending; }
        Poll::Ready(self.payload)
    }
}
impl Drop for Produced {
    fn drop(&mut self) { self.payload.fill(99); self.drops.fetch_add(1, Ordering::SeqCst); }
}
#[test]
fn await_result_survives_completed_future_drop() {
    let drops = Arc::new(AtomicUsize::new(0));
    let copy = drops.clone();
    let value = finish(async move {
        let value = Produced { pending: false, drops: copy.clone(), payload: [17; 4] }.await;
        assert_eq!(copy.load(Ordering::SeqCst), 1);
        let other = Produced { pending: false, drops: copy, payload: [23; 4] }.await;
        assert_eq!(other, [23; 4]);
        value
    });
    assert_eq!(value, [17; 4]);
    assert_eq!(drops.load(Ordering::SeqCst), 2);
}
