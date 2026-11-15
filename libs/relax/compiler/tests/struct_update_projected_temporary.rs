struct Leaf {
    label: String,
    bytes: Vec<u8>,
}
struct Pair {
    name: String,
    first: Leaf,
    second: Leaf,
}
impl Default for Pair {
    fn default() -> Self {
        Self {
            name: "base".into(),
            first: Leaf { label: "first".into(), bytes: vec![1, 2, 3] },
            second: Leaf { label: "second".into(), bytes: vec![4, 5, 6] },
        }
    }
}
#[test]
fn update_from_field_of_temporary_aggregate_keeps_unique_vector_ownership() {
    let value = Pair {
        first: Leaf { label: "changed".into(), ..Pair::default().first },
        ..Pair::default()
    };
    assert_eq!(value.name, "base");
    assert_eq!(value.first.label, "changed");
    assert_eq!(value.first.bytes.len(), 3);
    assert_eq!(value.first.bytes[0], 1);
    assert_eq!(value.second.bytes.len(), 3);
    assert_eq!(value.second.bytes[0], 4);
    drop(value);

    let log = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let updated = TracedLeaf {
        replaced: Traced(4, log.clone()),
        ..traced_parent(&log).leaf
    };
    assert_eq!(updated.kept.0, 1);
    assert_eq!(updated.replaced.0, 4);
    assert_eq!(log.borrow().as_slice(), &[2, 3]);
    drop(updated);
    assert_eq!(log.borrow().as_slice(), &[2, 3, 1, 4]);
}

struct Traced(u8, std::rc::Rc<std::cell::RefCell<Vec<u8>>>);
impl Drop for Traced {
    fn drop(&mut self) {
        self.1.borrow_mut().push(self.0);
    }
}
struct TracedLeaf { kept: Traced, replaced: Traced }
struct TracedParent { leaf: TracedLeaf, sibling: Traced }
fn traced_parent(log: &std::rc::Rc<std::cell::RefCell<Vec<u8>>>) -> TracedParent {
    let parent = TracedParent {
        leaf: TracedLeaf { kept: Traced(1, log.clone()), replaced: Traced(2, log.clone()) },
        sibling: Traced(3, log.clone()),
    };
    assert_eq!(parent.sibling.0, 3);
    parent
}
