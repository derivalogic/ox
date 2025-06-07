use std::cell::{Cell, RefCell};
use std::collections::HashSet;

use crate::prelude::*;

/// Visitor that records, for each `if` node, the indices of variables
/// modified inside the `if` block (including nested `if`s). It also
/// keeps track of the maximum nesting depth of `if` statements.
pub struct IfProcessor {
    var_stack: RefCell<Vec<HashSet<usize>>>,
    nested_if_lvl: Cell<usize>,
    max_nested_ifs: Cell<usize>,
}

impl IfProcessor {
    /// Create a new `IfProcessor` visitor.
    pub fn new() -> Self {
        Self {
            var_stack: RefCell::new(Vec::new()),
            nested_if_lvl: Cell::new(0),
            max_nested_ifs: Cell::new(0),
        }
    }

    /// Maximum nesting depth encountered after visiting.
    pub fn max_nested_ifs(&self) -> usize {
        self.max_nested_ifs.get()
    }

    /// Visit all events in a stream.
    pub fn visit_events(&self, events: &EventStream) -> Result<()> {
        for ev in events.events().iter() {
            self.visit(ev.expr())?;
        }
        Ok(())
    }
}

impl NodeVisitor for IfProcessor {
    type Output = Result<()>;

    fn visit(&self, node: &Box<Node>) -> Self::Output {
        match node.as_ref() {
            Node::If(children, _, affected, ..) => {
                let lvl = self.nested_if_lvl.get() + 1;
                self.nested_if_lvl.set(lvl);
                if lvl > self.max_nested_ifs.get() {
                    self.max_nested_ifs.set(lvl);
                }

                self.var_stack.borrow_mut().push(HashSet::new());

                for c in children.iter().skip(1) {
                    self.visit(c)?;
                }

                let vars = self.var_stack.borrow_mut().pop().unwrap();
                let mut vec: Vec<usize> = vars.iter().cloned().collect();
                vec.sort_unstable();
                affected.set(vec.clone()).ok();

                let lvl = lvl - 1;
                self.nested_if_lvl.set(lvl);
                if lvl > 0 {
                    let mut stack = self.var_stack.borrow_mut();
                    if let Some(top) = stack.last_mut() {
                        for v in vars {
                            top.insert(v);
                        }
                    }
                }
                Ok(())
            }
            Node::Assign(children) => {
                if self.nested_if_lvl.get() > 0 {
                    if let Some(var) = children.get(0) {
                        self.visit(var)?;
                    }
                }
                Ok(())
            }
            Node::Pays(children, _, _, _, _) => {
                if self.nested_if_lvl.get() > 0 {
                    if let Some(var) = children.get(0) {
                        self.visit(var)?;
                    }
                }
                Ok(())
            }
            Node::Variable(_, _, idx) => {
                if self.nested_if_lvl.get() > 0 {
                    if let Some(i) = idx.get() {
                        if let Some(top) = self.var_stack.borrow_mut().last_mut() {
                            top.insert(*i);
                        }
                    }
                }
                Ok(())
            }
            _ => {
                let children = node.children();
                for c in children.iter() {
                    self.visit(c)?;
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_if_processor_nested() {
        let script = "x = 0; if x == 0 { y = 1; if y == 1 { z = 2; } w = 3; }";
        let expr = ExprTree::try_from(script).unwrap();

        let indexer = EventIndexer::new();
        indexer.visit(&expr).unwrap();

        let processor = IfProcessor::new();
        processor.visit(&expr).unwrap();

        // outer if is second expression in base node
        let outer_if = match expr.as_ref() {
            Node::Base(children) => &children[1],
            _ => panic!("expected base node"),
        };

        let inner_if = match outer_if.as_ref() {
            Node::If(children, _, _, ..) => &children[2],
            _ => panic!("expected if node"),
        };

        if let Node::If(_, _, affected, ..) = outer_if.as_ref() {
            assert_eq!(affected.get().unwrap(), &vec![1, 2, 3]);
        } else {
            panic!("expected if node");
        }

        if let Node::If(_, _, affected, ..) = inner_if.as_ref() {
            assert_eq!(affected.get().unwrap(), &vec![2]);
        } else {
            panic!("expected inner if node");
        }

        assert_eq!(processor.max_nested_ifs(), 2);
    }
}
