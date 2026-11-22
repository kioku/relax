// Well-formedness of the self type does not excuse a mismatched signature.
// rejects: E0053
pub trait Value {
    fn value() -> u8;
}
pub struct Item;
impl Value for Item {
    fn value() -> u16 { 1 }
}
