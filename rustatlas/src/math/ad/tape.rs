use super::node::Node;
use std::cell::RefCell;

#[derive(Default)]
pub struct Tape {
    pub nodes: Vec<Node>,
    pub mark: usize,
}

impl Tape {
    pub fn record(&mut self, n: Node) -> usize {
        self.nodes.push(n);
        self.nodes.len() - 1
    }
    pub fn new_leaf(&mut self) -> usize {
        self.record(Node::default())
    }
}

thread_local! {
    pub static TAPE: RefCell<Tape> = RefCell::new(Tape::default());
}

/// Mark the current end of the tape (useful to propagate only a suffix)
pub fn set_mark() {
    TAPE.with(|t| t.borrow_mut().mark = t.borrow().nodes.len());
}

/// Full reverse sweep (every node, single adjoint)
pub fn propagate_all() {
    TAPE.with(|t| {
        let mut t = t.borrow_mut();
        for i in (0..t.nodes.len()).rev() {
            let n = t.nodes[i].clone();
            n.propagate_into(&mut t.nodes);
        }
    });
}

/* propagate_range() – inclusive indices, internal helper */
pub fn propagate_range(from: usize, to: usize) {
    if from < to {
        return;
    }
    TAPE.with(|t| {
        let mut t = t.borrow_mut();
        for i in (to..=from).rev() {
            let n = t.nodes[i].clone();
            n.propagate_into(&mut t.nodes);
        }
    });
}
