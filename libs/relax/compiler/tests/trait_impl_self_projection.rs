// A projection in an impl signature can normalize away the self type, but
// well-formedness of the impl header still supplies its outlives bounds.
use std::{marker::PhantomData, ptr::NonNull};

struct Header(usize);
struct Task<S: 'static>(PhantomData<S>);

trait Link {
    type Target;
}
impl<S> Link for Task<S> {
    type Target = Header;
}

trait Sharded: Link {
    fn shard_id(target: NonNull<Self::Target>) -> usize;
}
impl<S> Sharded for Task<S> {
    fn shard_id(target: NonNull<Self::Target>) -> usize {
        // SAFETY: The test passes a pointer borrowed from its live Header.
        unsafe { target.as_ref() }.0
    }
}

fn shard<L: Sharded>(target: NonNull<L::Target>) -> usize {
    L::shard_id(target)
}

#[test]
fn impl_self_supplies_the_projection_bound() {
    let header = Header(7);
    assert_eq!(shard::<Task<u32>>(NonNull::from(&header)), 7);
}
