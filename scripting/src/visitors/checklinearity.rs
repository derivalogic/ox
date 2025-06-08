// use std::cell::Cell;

// use crate::prelude::*;

// /// Visitor that checks whether an expression contains control flow
// /// or non-linear min/max constructs.
// ///
// /// A script is considered linear if it does not contain `if` statements
// /// and does not call `min` or `max`.
// pub struct CheckLinearity {
//     linear: Cell<bool>,
// }

// impl CheckLinearity {
//     /// Create a new `CheckLinearity` visitor.
//     pub fn new() -> Self {
//         Self {
//             linear: Cell::new(true),
//         }
//     }

//     /// Returns `true` if the visited script is linear.
//     pub fn is_linear(&self) -> bool {
//         self.linear.get()
//     }

//     /// Visit a stream of events and return whether it is linear.
//     pub fn visit_events(&self, events: &EventStream) -> bool {
//         events.events().iter().for_each(|ev| self.visit(ev.expr()));
//         self.is_linear()
//     }
// }

// impl NodeVisitor for CheckLinearity {
//     type Output = ();

//     fn visit(&self, node: &Box<Node>) {
//         match node.as_ref() {
//             Node::If(..) | Node::Min(_) | Node::Max(_) => {
//                 self.linear.set(false);
//             }
//             _ => {}
//         }

//         match node.as_ref() {
//             Node::Base(children)
//             | Node::Add(children)
//             | Node::Subtract(children)
//             | Node::Multiply(children)
//             | Node::Divide(children)
//             | Node::Assign(children)
//             | Node::Min(children)
//             | Node::Max(children)
//             | Node::Exp(children)
//             | Node::Pow(children)
//             | Node::Ln(children)
//             | Node::Fif(children)
//             | Node::Cvg(children)
//             | Node::Append(children)
//             | Node::Mean(children)
//             | Node::Std(children)
//             | Node::Index(children)
//             | Node::UnaryPlus(children)
//             | Node::UnaryMinus(children)
//             | Node::Equal(children)
//             | Node::NotEqual(children)
//             | Node::And(children)
//             | Node::Or(children)
//             | Node::Not(children)
//             | Node::Superior(children)
//             | Node::Inferior(children)
//             | Node::SuperiorOrEqual(children)
//             | Node::InferiorOrEqual(children)
//             | Node::If(children, ..)
//             | Node::Pays(children, _, _, _, _)
//             | Node::Range(children)
//             | Node::List(children)
//             | Node::Variable(children, ..) => {
//                 children.iter().for_each(|c| self.visit(c));
//             }
//             Node::ForEach(_, iter, body, _) => {
//                 self.visit(iter);
//                 body.iter().for_each(|c| self.visit(c));
//             }
//             Node::Spot(_, _, _, _)
//             | Node::Df(_, _, _)
//             | Node::RateIndex(_, _, _, _)
//             | Node::True
//             | Node::False
//             | Node::Constant(_)
//             | Node::String(_) => {}
//         }
//     }
// }

// #[cfg(test)]
// mod ai_gen_tests {
//     use super::*;

//     #[test]
//     fn test_linear_script() {
//         let script = "x = 1; y = 2; z = x + y;";
//         let expr = ExprTree::try_from(script).unwrap();
//         let checker = CheckLinearity::new();
//         checker.visit(&expr);
//         assert!(checker.is_linear());
//     }

//     #[test]
//     fn test_if_makes_nonlinear() {
//         let script = "x = 1; if (x > 0) { y = 1; }";
//         let expr = ExprTree::try_from(script).unwrap();
//         let checker = CheckLinearity::new();
//         checker.visit(&expr);
//         assert!(!checker.is_linear());
//     }

//     #[test]
//     fn test_min_makes_nonlinear() {
//         let script = "x = min(1, 2);";
//         let expr = ExprTree::try_from(script).unwrap();
//         let checker = CheckLinearity::new();
//         checker.visit(&expr);
//         assert!(!checker.is_linear());
//     }
// }
