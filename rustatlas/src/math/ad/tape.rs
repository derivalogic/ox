use crate::prelude::*;
use std::cell::RefCell;

#[derive(Default)]
pub struct Tape {
    nodes: Vec<TapeNode>,
    mark: usize,
    pub active: bool,
}

impl Tape {
    pub fn record(&mut self, n: TapeNode) -> Option<usize> {
        match self.active {
            true => {
                self.nodes.push(n);
                Some(self.nodes.len() - 1)
            }
            false => None,
        }
    }
    pub fn new_leaf(&mut self) -> Option<usize> {
        match self.active {
            true => self.record(TapeNode::default()),
            false => None,
        }
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn node(&self, idx: usize) -> Result<&TapeNode> {
        if idx < self.nodes.len() {
            Ok(&self.nodes[idx])
        } else {
            Err(AtlasError::NodeNotInTapeErr(idx))
        }
    }

    pub fn mut_node(&mut self, idx: usize) -> Result<&mut TapeNode> {
        if idx < self.nodes.len() {
            Ok(&mut self.nodes[idx])
        } else {
            Err(AtlasError::NodeNotInTapeErr(idx))
        }
    }

    pub fn mark(&self) -> usize {
        self.mark
    }

    pub fn start_recording() {
        TAPE.with(|t| {
            let mut t = t.borrow_mut();
            t.nodes.clear();
            t.mark = 0;
            t.set_active(true);
        });
    }

    pub fn is_active() -> bool {
        TAPE.with(|t| t.borrow().active)
    }

    /// Mark the current end of the tape (useful to propagate only a suffix)
    pub fn set_mark() {
        TAPE.with(|t| t.borrow_mut().mark = t.borrow().nodes.len());
    }

    pub fn rewind_to_mark() {
        TAPE.with(|t| {
            let mark = t.borrow().mark;
            let mut t = t.borrow_mut();
            t.nodes.truncate(mark);
        });
    }

    pub fn rewind_to_init() {
        TAPE.with(|t| {
            let mut t = t.borrow_mut();
            t.nodes.clear();
        });
    }

    pub fn reset_adjoints() {
        TAPE.with(|t| {
            for n in &mut t.borrow_mut().nodes {
                n.adj = 0.0;
            }
        });
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
}

thread_local! {
    pub static TAPE: RefCell<Tape> = RefCell::new(Tape::default());
}
