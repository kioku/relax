// A projection that normalizes to () does not imply bounds from its arguments.
// rejects: lifetime may not live long enough
pub trait Erase {
    type Output;
}
impl<T> Erase for T {
    type Output = ();
}
pub fn extend<'a, 'b>(_: <&'a &'b () as Erase>::Output, value: &'b u8) -> &'a u8 {
    value
}
