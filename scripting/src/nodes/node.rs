use std::sync::OnceLock;

use rustatlas::prelude::*;

use crate::prelude::*;

pub type ExprTree = Box<Node>;

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Base(Vec<ExprTree>),

    // variables
    Variable(Vec<ExprTree>, String, OnceLock<usize>),
    Constant(f64),
    String(String),

    // financial
    Spot(Currency, OnceLock<usize>),
    Pays(Vec<ExprTree>, OnceLock<usize>),

    // math
    Add(Vec<ExprTree>),
    Subtract(Vec<ExprTree>),
    Multiply(Vec<ExprTree>),
    Divide(Vec<ExprTree>),
    Assign(Vec<ExprTree>),
    Min(Vec<ExprTree>),
    Max(Vec<ExprTree>),
    Exp(Vec<ExprTree>),
    Pow(Vec<ExprTree>),
    Ln(Vec<ExprTree>),

    // unary
    UnaryPlus(Vec<ExprTree>),
    UnaryMinus(Vec<ExprTree>),

    // logic
    True,
    False,
    Equal(Vec<ExprTree>),
    NotEqual(Vec<ExprTree>),
    And(Vec<ExprTree>),
    Or(Vec<ExprTree>),
    Not(Vec<ExprTree>),
    Superior(Vec<ExprTree>),
    Inferior(Vec<ExprTree>),
    SuperiorOrEqual(Vec<ExprTree>),
    InferiorOrEqual(Vec<ExprTree>),

    // control flow
    If(Vec<ExprTree>, Option<usize>),
}

impl Node {
    pub fn new_base() -> Node {
        Node::Base(Vec::new())
    }

    pub fn new_add() -> Node {
        Node::Add(Vec::new())
    }

    pub fn new_subtract() -> Node {
        Node::Subtract(Vec::new())
    }

    pub fn new_multiply() -> Node {
        Node::Multiply(Vec::new())
    }

    pub fn new_divide() -> Node {
        Node::Divide(Vec::new())
    }

    pub fn new_variable(name: String) -> Node {
        Node::Variable(Vec::new(), name, OnceLock::new())
    }

    pub fn new_variable_with_id(name: String, id: usize) -> Node {
        Node::Variable(Vec::new(), name, id.into())
    }

    pub fn new_min() -> Node {
        Node::Min(Vec::new())
    }

    pub fn new_max() -> Node {
        Node::Max(Vec::new())
    }

    pub fn new_exp() -> Node {
        Node::Exp(Vec::new())
    }

    pub fn new_ln() -> Node {
        Node::Ln(Vec::new())
    }

    pub fn new_pow() -> Node {
        Node::Pow(Vec::new())
    }

    pub fn new_constant(value: f64) -> Node {
        Node::Constant(value)
    }

    pub fn new_assign() -> Node {
        Node::Assign(Vec::new())
    }

    pub fn new_and() -> Node {
        Node::And(Vec::new())
    }

    pub fn new_or() -> Node {
        Node::Or(Vec::new())
    }

    pub fn new_not() -> Node {
        Node::Not(Vec::new())
    }

    pub fn new_superior() -> Node {
        Node::Superior(Vec::new())
    }

    pub fn new_inferior() -> Node {
        Node::Inferior(Vec::new())
    }

    pub fn new_superior_or_equal() -> Node {
        Node::SuperiorOrEqual(Vec::new())
    }

    pub fn new_equal() -> Node {
        Node::Equal(Vec::new())
    }

    pub fn new_if() -> Node {
        Node::If(Vec::new(), None)
    }

    pub fn new_unary_plus() -> Node {
        Node::UnaryPlus(Vec::new())
    }

    pub fn new_unary_minus() -> Node {
        Node::UnaryMinus(Vec::new())
    }

    pub fn new_inferior_or_equal() -> Node {
        Node::InferiorOrEqual(Vec::new())
    }

    pub fn new_not_equal() -> Node {
        Node::NotEqual(Vec::new())
    }

    pub fn new_true() -> Node {
        Node::True
    }

    pub fn new_false() -> Node {
        Node::False
    }

    pub fn new_pays() -> Node {
        Node::Pays(Vec::new(), OnceLock::new())
    }

    pub fn add_child(&mut self, child: ExprTree) {
        match self {
            Node::Base(children) => children.push(child),
            Node::Add(children) => children.push(child),
            Node::Subtract(children) => children.push(child),
            Node::Multiply(children) => children.push(child),
            Node::Divide(children) => children.push(child),
            Node::Variable(children, _, _) => children.push(child),
            Node::Assign(children) => children.push(child),
            Node::And(children) => children.push(child),
            Node::Or(children) => children.push(child),
            Node::Not(children) => children.push(child),
            Node::Superior(children) => children.push(child),
            Node::Inferior(children) => children.push(child),
            Node::SuperiorOrEqual(children) => children.push(child),
            Node::InferiorOrEqual(children) => children.push(child),
            Node::Equal(children) => children.push(child),
            Node::If(children, _) => children.push(child),
            Node::UnaryPlus(children) => children.push(child),
            Node::UnaryMinus(children) => children.push(child),
            Node::Min(children) => children.push(child),
            Node::Max(children) => children.push(child),
            Node::Exp(children) => children.push(child),
            Node::Ln(children) => children.push(child),
            Node::Pow(children) => children.push(child),
            Node::NotEqual(children) => children.push(child),
            Node::Pays(children, _) => children.push(child),
            Node::Spot(_, _) => panic!("Cannot add child to spot node"),
            Node::True => panic!("Cannot add child to true node"),
            Node::False => panic!("Cannot add child to false node"),
            Node::Constant(_) => panic!("Cannot add child to constant node"),
            Node::String(_) => panic!("Cannot add child to string node"),
        }
    }

    pub fn children(&self) -> &Vec<ExprTree> {
        match self {
            Node::Base(children) => children,
            Node::Add(children) => children,
            Node::Subtract(children) => children,
            Node::Multiply(children) => children,
            Node::Divide(children) => children,
            Node::Variable(children, _, _) => children,
            Node::Assign(children) => children,
            Node::And(children) => children,
            Node::Or(children) => children,
            Node::Not(children) => children,
            Node::Superior(children) => children,
            Node::Inferior(children) => children,
            Node::SuperiorOrEqual(children) => children,
            Node::InferiorOrEqual(children) => children,
            Node::Equal(children) => children,
            Node::If(children, _) => children,
            Node::UnaryPlus(children) => children,
            Node::UnaryMinus(children) => children,
            Node::Min(children) => children,
            Node::Max(children) => children,
            Node::Exp(children) => children,
            Node::Ln(children) => children,
            Node::Pow(children) => children,
            Node::NotEqual(children) => children,
            Node::Pays(children, _) => children,
            Node::Spot(_, _) => panic!("Cannot get children from spot node"),
            Node::True => panic!("Cannot get children from true node"),
            Node::False => panic!("Cannot get children from false node"),
            Node::Constant(_) => panic!("Cannot get children from constant node"),
            Node::String(_) => panic!("Cannot get children from string node"),
        }
    }
}

impl Visitable for Box<Node> {
    type Output = ();
    fn accept(&mut self, visitor: &impl NodeVisitor) {
        visitor.visit(self);
    }
}

impl ConstVisitable for Box<Node> {
    type Output = ();
    fn const_accept(&self, visitor: &impl NodeConstVisitor) {
        visitor.const_visit(self.clone());
    }
}

#[cfg(test)]
mod ai_gen_tests {
    use super::*;

    #[test]
    fn test_new_base() {
        // Test the creation of a new base node
        let node = Node::new_base();
        assert_eq!(node, Node::Base(Vec::new()));
    }

    #[test]
    fn test_new_add() {
        // Test the creation of a new add node
        let node = Node::new_add();
        assert_eq!(node, Node::Add(Vec::new()));
    }

    #[test]
    fn test_new_subtract() {
        // Test the creation of a new subtract node
        let node = Node::new_subtract();
        assert_eq!(node, Node::Subtract(Vec::new()));
    }

    #[test]
    fn test_new_multiply() {
        // Test the creation of a new multiply node
        let node = Node::new_multiply();
        assert_eq!(node, Node::Multiply(Vec::new()));
    }

    #[test]
    fn test_new_divide() {
        // Test the creation of a new divide node
        let node = Node::new_divide();
        assert_eq!(node, Node::Divide(Vec::new()));
    }

    #[test]
    fn test_new_variable() {
        // Test the creation of a new variable node
        let node = Node::new_variable("x".to_string());
        assert_eq!(
            node,
            Node::Variable(Vec::new(), "x".to_string(), OnceLock::new())
        );
    }

    #[test]
    fn test_new_variable_with_id() {
        // Test the creation of a new variable node with an id
        let node = Node::new_variable_with_id("x".to_string(), 42);
        assert_eq!(node, Node::Variable(Vec::new(), "x".to_string(), 42.into()));
    }

    #[test]
    fn test_new_min() {
        // Test the creation of a new min node
        let node = Node::new_min();
        assert_eq!(node, Node::Min(Vec::new()));
    }

    #[test]
    fn test_new_max() {
        // Test the creation of a new max node
        let node = Node::new_max();
        assert_eq!(node, Node::Max(Vec::new()));
    }

    #[test]
    fn test_new_exp() {
        // Test the creation of a new exp node
        let node = Node::new_exp();
        assert_eq!(node, Node::Exp(Vec::new()));
    }

    #[test]
    fn test_new_ln() {
        // Test the creation of a new ln node
        let node = Node::new_ln();
        assert_eq!(node, Node::Ln(Vec::new()));
    }

    #[test]
    fn test_new_pow() {
        // Test the creation of a new pow node
        let node = Node::new_pow();
        assert_eq!(node, Node::Pow(Vec::new()));
    }

    #[test]
    fn test_new_constant() {
        // Test the creation of a new constant node
        let node = Node::new_constant(3.14);
        assert_eq!(node, Node::Constant(3.14));
    }

    #[test]
    fn test_new_assign() {
        // Test the creation of a new assign node
        let node = Node::new_assign();
        assert_eq!(node, Node::Assign(Vec::new()));
    }

    #[test]
    fn test_new_and() {
        // Test the creation of a new and node
        let node = Node::new_and();
        assert_eq!(node, Node::And(Vec::new()));
    }

    #[test]
    fn test_new_or() {
        // Test the creation of a new or node
        let node = Node::new_or();
        assert_eq!(node, Node::Or(Vec::new()));
    }

    #[test]
    fn test_new_not() {
        // Test the creation of a new not node
        let node = Node::new_not();
        assert_eq!(node, Node::Not(Vec::new()));
    }

    #[test]
    fn test_new_superior() {
        // Test the creation of a new superior node
        let node = Node::new_superior();
        assert_eq!(node, Node::Superior(Vec::new()));
    }

    #[test]
    fn test_new_inferior() {
        // Test the creation of a new inferior node
        let node = Node::new_inferior();
        assert_eq!(node, Node::Inferior(Vec::new()));
    }

    #[test]
    fn test_new_superior_or_equal() {
        // Test the creation of a new superior or equal node
        let node = Node::new_superior_or_equal();
        assert_eq!(node, Node::SuperiorOrEqual(Vec::new()));
    }

    #[test]
    fn test_new_equal() {
        // Test the creation of a new equal node
        let node = Node::new_equal();
        assert_eq!(node, Node::Equal(Vec::new()));
    }

    #[test]
    fn test_new_if() {
        // Test the creation of a new if node
        let node = Node::new_if();
        assert_eq!(node, Node::If(Vec::new(), None));
    }

    #[test]
    fn test_new_unary_plus() {
        // Test the creation of a new unary plus node
        let node = Node::new_unary_plus();
        assert_eq!(node, Node::UnaryPlus(Vec::new()));
    }

    #[test]
    fn test_new_unary_minus() {
        // Test the creation of a new unary minus node
        let node = Node::new_unary_minus();
        assert_eq!(node, Node::UnaryMinus(Vec::new()));
    }

    #[test]
    fn test_new_inferior_or_equal() {
        // Test the creation of a new inferior or equal node
        let node = Node::new_inferior_or_equal();
        assert_eq!(node, Node::InferiorOrEqual(Vec::new()));
    }

    #[test]
    fn test_new_not_equal() {
        // Test the creation of a new not equal node
        let node = Node::new_not_equal();
        assert_eq!(node, Node::NotEqual(Vec::new()));
    }

    #[test]
    fn test_new_true() {
        // Test the creation of a new true node
        let node = Node::new_true();
        assert_eq!(node, Node::True);
    }

    #[test]
    fn test_new_false() {
        // Test the creation of a new false node
        let node = Node::new_false();
        assert_eq!(node, Node::False);
    }

    #[test]
    fn test_new_pays() {
        // Test the creation of a new pays node
        let node = Node::new_pays();
        assert_eq!(node, Node::Pays(Vec::new(), OnceLock::new()));
    }

    #[test]
    fn test_add_child_to_base() {
        // Test adding a child to a base node
        let mut node = Node::new_base();
        let child = Box::new(Node::new_add());
        node.add_child(child.clone());
        assert_eq!(node.children(), &vec![child]);
    }

    #[test]
    fn test_add_child_to_add() {
        // Test adding a child to an add node
        let mut node = Node::new_add();
        let child = Box::new(Node::new_subtract());
        node.add_child(child.clone());
        assert_eq!(node.children(), &vec![child]);
    }

    #[test]
    #[should_panic(expected = "Cannot add child to spot node")]
    fn test_add_child_to_spot() {
        // Test adding a child to a spot node, which should panic
        let mut node = Node::Spot(Currency::USD, OnceLock::new());
        let child = Box::new(Node::new_add());
        node.add_child(child);
    }

    #[test]
    #[should_panic(expected = "Cannot add child to true node")]
    fn test_add_child_to_true() {
        // Test adding a child to a true node, which should panic
        let mut node = Node::True;
        let child = Box::new(Node::new_add());
        node.add_child(child);
    }

    #[test]
    #[should_panic(expected = "Cannot add child to constant node")]
    fn test_add_child_to_constant() {
        // Test adding a child to a constant node, which should panic
        let mut node = Node::Constant(3.14);
        let child = Box::new(Node::new_add());
        node.add_child(child);
    }

    #[test]
    fn test_children_of_base() {
        // Test getting children of a base node
        let mut node = Node::new_base();
        let child = Box::new(Node::new_add());
        node.add_child(child.clone());
        assert_eq!(node.children(), &vec![child]);
    }

    #[test]
    #[should_panic(expected = "Cannot get children from spot node")]
    fn test_children_of_spot() {
        // Test getting children of a spot node, which should panic
        let node = Node::Spot(Currency::USD, OnceLock::new());
        node.children();
    }

    #[test]
    #[should_panic(expected = "Cannot get children from true node")]
    fn test_children_of_true() {
        // Test getting children of a true node, which should panic
        let node = Node::True;
        node.children();
    }

    #[test]
    #[should_panic(expected = "Cannot get children from constant node")]
    fn test_children_of_constant() {
        // Test getting children of a constant node, which should panic
        let node = Node::Constant(3.14);
        node.children();
    }
}
