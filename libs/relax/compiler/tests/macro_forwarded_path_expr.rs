#[derive(Debug, PartialEq)]
enum Outcome {
    Empty,
}

macro_rules! assert_path_value {
    ($expected:path) => {
        assert_eq!(Outcome::Empty, $expected);
    };
}

macro_rules! opaque_path {
    (Outcome::Empty) => { false };
    ($other:path) => { true };
}

macro_rules! forward_path {
    ($value:path) => { opaque_path!($value) };
}

#[test]
fn path_forwarded_as_expression_keeps_its_ast_and_opacity() {
    assert_path_value!(Outcome::Empty);
    assert!(forward_path!(Outcome::Empty));
}
