use crate::prelude::*;
use rustatlas::prelude::*;

// pub type ExprTree = Box<Node>;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct NodeData {
    pub children: Vec<Node>,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct BoolData {
    pub always_true: bool,
    pub always_false: bool,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CompData {
    pub bool_sub_node: BoolData,
    pub discrete: bool,
    pub eps: f64,
    pub lb: f64,
    pub rb: f64,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ExprData {
    pub children: Vec<Node>,
    pub is_constant: bool,
    pub const_value: f64,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct VarData {
    pub name: String,
    pub id: Option<usize>,
    pub expr_data: ExprData,
    pub bool_data: BoolData,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct IfData {
    pub first_else: Option<usize>,
    pub affected_vars: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpotData {
    pub first: Currency,
    pub second: Currency,
    pub date: Option<Date>,
    pub id: Option<usize>,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct DfData {
    pub date: Date,
    pub curve: Option<String>,
    pub id: Option<usize>,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct RateIndexData {
    pub name: String,
    pub start: Date,
    pub end: Date,
    pub id: Option<usize>,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct PaysData {
    pub children: Vec<Node>,
    pub date: Option<Date>,
    pub currency: Option<Currency>,
    pub id: Option<usize>,
    pub index_id: Option<usize>,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ForEachData {
    pub var: String,
    pub id: Option<usize>,
    pub node: Box<Node>,
    pub iter: Box<Vec<Node>>,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum Node {
    Base(NodeData),

    // variables
    Variable(VarData),
    Constant(VarData),
    String(String),

    // financial
    Spot(SpotData),
    Df(DfData),
    RateIndex(RateIndexData),
    Pays(PaysData),

    // math
    Add(NodeData),
    Subtract(NodeData),
    Multiply(NodeData),
    Divide(NodeData),
    Assign(NodeData),
    Min(NodeData),
    Max(NodeData),
    Exp(NodeData),
    Pow(NodeData),
    Ln(NodeData),
    Fif(NodeData),
    Cvg(NodeData),
    Append(NodeData),
    Mean(NodeData),
    Std(NodeData),
    Index(NodeData),

    // unary
    UnaryPlus(NodeData),
    UnaryMinus(NodeData),

    // logic
    #[default]
    True,
    False,

    Equal(NodeData),
    NotEqual(NodeData),
    And(NodeData),
    Or(NodeData),
    Not(NodeData),
    Superior(NodeData),
    Inferior(NodeData),
    SuperiorOrEqual(NodeData),
    InferiorOrEqual(NodeData),

    // control flow
    If(IfData),
    ForEach(ForEachData),

    // iterable
    Range(NodeData),
    List(NodeData),
}

impl Node {
    pub fn new_base() -> Node {
        Node::Base(NodeData::default())
    }

    pub fn new_add() -> Node {
        Node::Add(NodeData::default())
    }

    pub fn new_subtract() -> Node {
        Node::Subtract(NodeData::default())
    }

    pub fn new_multiply() -> Node {
        Node::Multiply(NodeData::default())
    }

    pub fn new_divide() -> Node {
        Node::Divide(NodeData::default())
    }

    pub fn new_variable(name: String) -> Node {
        Node::Variable(VarData {
            name,
            id: None,
            expr_data: ExprData::default(),
            bool_data: BoolData::default(),
        })
    }

    pub fn new_variable_with_id(name: String, id: usize) -> Node {
        Node::Variable(VarData {
            name,
            id: Some(id),
            expr_data: ExprData::default(),
            bool_data: BoolData::default(),
        })
    }

    pub fn new_min() -> Node {
        Node::Min(NodeData::default())
    }

    pub fn new_max() -> Node {
        Node::Max(NodeData::default())
    }

    pub fn new_exp() -> Node {
        Node::Exp(NodeData::default())
    }

    pub fn new_ln() -> Node {
        Node::Ln(NodeData::default())
    }

    pub fn new_fif() -> Node {
        Node::Fif(NodeData::default())
    }

    pub fn new_pow() -> Node {
        Node::Pow(NodeData::default())
    }

    pub fn new_cvg() -> Node {
        Node::Cvg(NodeData::default())
    }

    pub fn new_append() -> Node {
        Node::Append(NodeData::default())
    }

    pub fn new_mean() -> Node {
        Node::Mean(NodeData::default())
    }

    pub fn new_std() -> Node {
        Node::Std(NodeData::default())
    }

    pub fn new_index() -> Node {
        Node::Index(NodeData::default())
    }

    pub fn new_constant(value: NumericType) -> Node {
        Node::Constant(VarData {
            name: value.to_string(),
            id: None,
            expr_data: ExprData {
                children: Vec::new(),
                is_constant: true,
                const_value: value.value(),
            },
            bool_data: BoolData::default(),
        })
    }

    pub fn new_assign() -> Node {
        Node::Assign(NodeData::default())
    }

    pub fn new_and() -> Node {
        Node::And(NodeData::default())
    }

    pub fn new_or() -> Node {
        Node::Or(NodeData::default())
    }

    pub fn new_not() -> Node {
        Node::Not(NodeData::default())
    }

    pub fn new_superior() -> Node {
        Node::Superior(NodeData::default())
    }

    pub fn new_inferior() -> Node {
        Node::Inferior(NodeData::default())
    }

    pub fn new_superior_or_equal() -> Node {
        Node::SuperiorOrEqual(NodeData::default())
    }

    pub fn new_equal() -> Node {
        Node::Equal(NodeData::default())
    }

    pub fn new_if() -> Node {
        Node::If(IfData::default())
    }

    pub fn new_unary_plus() -> Node {
        Node::UnaryPlus(NodeData::default())
    }

    pub fn new_unary_minus() -> Node {
        Node::UnaryMinus(NodeData::default())
    }

    pub fn new_inferior_or_equal() -> Node {
        Node::InferiorOrEqual(NodeData::default())
    }

    pub fn new_not_equal() -> Node {
        Node::NotEqual(NodeData::default())
    }

    pub fn new_true() -> Node {
        Node::True
    }

    pub fn new_false() -> Node {
        Node::False
    }

    pub fn new_pays() -> Node {
        Node::Pays(PaysData::default())
    }

    pub fn new_spot(first: Currency, second: Currency, date: Option<Date>) -> Node {
        Node::Spot(SpotData {
            first,
            second,
            date,
            id: None,
        })
    }

    pub fn new_df(date: Date, curve: Option<String>) -> Node {
        Node::Df(DfData {
            date,
            curve,
            id: None,
        })
    }

    pub fn new_rate_index(name: String, start: Date, end: Date) -> Node {
        Node::RateIndex(RateIndexData {
            name,
            start,
            end,
            id: None,
        })
    }

    pub fn new_range() -> Node {
        Node::Range(NodeData::default())
    }

    pub fn new_list() -> Node {
        Node::List(NodeData::default())
    }

    pub fn new_for_each(var: String, node: Box<Node>, iter: Box<Vec<Node>>) -> Node {
        Node::ForEach(ForEachData {
            var,
            iter: iter,
            node: node,
            id: None,
        })
    }

    pub fn add_child(&mut self, child: Node) {
        match self {
            Node::Base(inner) => inner.children.push(child),
            Node::Add(data) => data.children.push(child),
            Node::Subtract(data) => data.children.push(child),
            Node::Multiply(data) => data.children.push(child),
            Node::Divide(data) => data.children.push(child),
            Node::Variable(data) => data.expr_data.children.push(child),
            Node::Assign(data) => data.children.push(child),
            Node::And(data) => data.children.push(child),
            Node::Or(data) => data.children.push(child),
            Node::Not(data) => data.children.push(child),
            Node::Superior(data) => data.children.push(child),
            Node::Inferior(data) => data.children.push(child),
            Node::SuperiorOrEqual(data) => data.children.push(child),
            Node::InferiorOrEqual(data) => data.children.push(child),
            Node::Equal(data) => data.children.push(child),
            Node::If(_) => panic!("Cannot add child to if node directly"),
            Node::UnaryPlus(data) => data.children.push(child),
            Node::UnaryMinus(data) => data.children.push(child),
            Node::Min(data) => data.children.push(child),
            Node::Max(data) => data.children.push(child),
            Node::Exp(data) => data.children.push(child),
            Node::Ln(data) => data.children.push(child),
            Node::Fif(data) => data.children.push(child),
            Node::Pow(data) => data.children.push(child),
            Node::Cvg(data) => data.children.push(child),
            Node::Append(data) => data.children.push(child),
            Node::Mean(data) => data.children.push(child),
            Node::Std(data) => data.children.push(child),
            Node::Index(data) => data.children.push(child),
            Node::NotEqual(data) => data.children.push(child),
            Node::Pays(data) => data.children.push(child),
            Node::ForEach(data) => data.node.add_child(child),
            Node::Range(data) => data.children.push(child),
            Node::List(data) => data.children.push(child),
            Node::Spot(_) => panic!("Cannot add child to spot node"),
            Node::Df(_) => panic!("Cannot add child to df node"),
            Node::RateIndex(_) => panic!("Cannot add child to rate index node"),
            Node::True => panic!("Cannot add child to true node"),
            Node::False => panic!("Cannot add child to false node"),
            Node::Constant(_) => panic!("Cannot add child to constant node"),
            Node::String(_) => panic!("Cannot add child to string node"),
        }
    }

    pub fn children(&self) -> &Vec<Node> {
        match self {
            Node::Base(data) => &data.children,
            Node::Add(data) => &data.children,
            Node::Subtract(data) => &data.children,
            Node::Multiply(data) => &data.children,
            Node::Divide(data) => &data.children,
            Node::Variable(data) => &data.expr_data.children,
            Node::Assign(data) => &data.children,
            Node::And(data) => &data.children,
            Node::Or(data) => &data.children,
            Node::Not(data) => &data.children,
            Node::Superior(data) => &data.children,
            Node::Inferior(data) => &data.children,
            Node::SuperiorOrEqual(data) => &data.children,
            Node::InferiorOrEqual(data) => &data.children,
            Node::Equal(data) => &data.children,
            Node::If(_) => panic!("Cannot get children from if node directly"),
            Node::UnaryPlus(data) => &data.children,
            Node::UnaryMinus(data) => &data.children,
            Node::Min(data) => &data.children,
            Node::Max(data) => &data.children,
            Node::Exp(data) => &data.children,
            Node::Ln(data) => &data.children,
            Node::Fif(data) => &data.children,
            Node::Pow(data) => &data.children,
            Node::Cvg(data) => &data.children,
            Node::Append(data) => &data.children,
            Node::Mean(data) => &data.children,
            Node::Std(data) => &data.children,
            Node::Index(data) => &data.children,
            Node::NotEqual(data) => &data.children,
            Node::Pays(data) => &data.children,
            Node::ForEach(data) => panic!("Cannot get children from foreach node directly"),
            Node::Range(data) => &data.children,
            Node::List(data) => &data.children,
            Node::Spot(_) => panic!("Cannot get children from spot node"),
            Node::Df(_) => panic!("Cannot get children from df node"),
            Node::RateIndex(_) => {
                panic!("Cannot get children from rate index node")
            }
            Node::True => panic!("Cannot get children from true node"),
            Node::False => panic!("Cannot get children from false node"),
            Node::Constant(_) => panic!("Cannot get children from constant node"),
            Node::String(_) => panic!("Cannot get children from string node"),
        }
    }
}

impl Visitable for Node {
    type Output = ();
    fn accept(&mut self, visitor: &impl NodeVisitor) {
        visitor.visit(self);
    }
}

impl ConstVisitable for Node {
    type Output = ();
    fn const_accept(&self, visitor: &impl NodeConstVisitor) {
        visitor.const_visit(self);
    }
}

// #[cfg(test)]
// mod ai_gen_tests {

//     use super::*;

//     #[test]
//     fn test_new_base() {
//         // Test the creation of a new base node
//         let node = Node::new_base();
//         assert_eq!(node, Node::Base(Vec::new()));
//     }

//     #[test]
//     fn test_new_add() {
//         // Test the creation of a new add node
//         let node = Node::new_add();
//         assert_eq!(node, Node::Add(Vec::new()));
//     }

//     #[test]
//     fn test_new_subtract() {
//         // Test the creation of a new subtract node
//         let node = Node::new_subtract();
//         assert_eq!(node, Node::Subtract(Vec::new()));
//     }

//     #[test]
//     fn test_new_multiply() {
//         // Test the creation of a new multiply node
//         let node = Node::new_multiply();
//         assert_eq!(node, Node::Multiply(Vec::new()));
//     }

//     #[test]
//     fn test_new_divide() {
//         // Test the creation of a new divide node
//         let node = Node::new_divide();
//         assert_eq!(node, Node::Divide(Vec::new()));
//     }

//     #[test]
//     fn test_new_variable() {
//         // Test the creation of a new variable node
//         let node = Node::new_variable("x".to_string());
//         assert_eq!(
//             node,
//             Node::Variable(Vec::new(), "x".to_string(), OnceLock::new())
//         );
//     }

//     #[test]
//     fn test_new_variable_with_id() {
//         // Test the creation of a new variable node with an id
//         let node = Node::new_variable_with_id("x".to_string(), 42);
//         assert_eq!(node, Node::Variable(Vec::new(), "x".to_string(), 42.into()));
//     }

//     #[test]
//     fn test_new_min() {
//         // Test the creation of a new min node
//         let node = Node::new_min();
//         assert_eq!(node, Node::Min(Vec::new()));
//     }

//     #[test]
//     fn test_new_max() {
//         // Test the creation of a new max node
//         let node = Node::new_max();
//         assert_eq!(node, Node::Max(Vec::new()));
//     }

//     #[test]
//     fn test_new_exp() {
//         // Test the creation of a new exp node
//         let node = Node::new_exp();
//         assert_eq!(node, Node::Exp(Vec::new()));
//     }

//     #[test]
//     fn test_new_ln() {
//         // Test the creation of a new ln node
//         let node = Node::new_ln();
//         assert_eq!(node, Node::Ln(Vec::new()));
//     }

//     #[test]
//     fn test_new_fif() {
//         // Test the creation of a new fif node
//         let node = Node::new_fif();
//         assert_eq!(node, Node::Fif(Vec::new()));
//     }

//     #[test]
//     fn test_new_pow() {
//         // Test the creation of a new pow node
//         let node = Node::new_pow();
//         assert_eq!(node, Node::Pow(Vec::new()));
//     }

//     #[test]
//     fn test_new_cvg() {
//         // Test the creation of a new cvg node
//         let node = Node::new_cvg();
//         assert_eq!(node, Node::Cvg(Vec::new()));
//     }

//     #[test]
//     fn test_new_constant() {
//         // Test the creation of a new constant node
//         let node = Node::new_constant(NumericType::new(3.14));
//         assert_eq!(node, Node::Constant(NumericType::new(3.14)));
//     }

//     #[test]
//     fn test_new_assign() {
//         // Test the creation of a new assign node
//         let node = Node::new_assign();
//         assert_eq!(node, Node::Assign(Vec::new()));
//     }

//     #[test]
//     fn test_new_and() {
//         // Test the creation of a new and node
//         let node = Node::new_and();
//         assert_eq!(node, Node::And(Vec::new()));
//     }

//     #[test]
//     fn test_new_or() {
//         // Test the creation of a new or node
//         let node = Node::new_or();
//         assert_eq!(node, Node::Or(Vec::new()));
//     }

//     #[test]
//     fn test_new_not() {
//         // Test the creation of a new not node
//         let node = Node::new_not();
//         assert_eq!(node, Node::Not(Vec::new()));
//     }

//     #[test]
//     fn test_new_superior() {
//         // Test the creation of a new superior node
//         let node = Node::new_superior();
//         assert_eq!(node, Node::Superior(Vec::new()));
//     }

//     #[test]
//     fn test_new_inferior() {
//         // Test the creation of a new inferior node
//         let node = Node::new_inferior();
//         assert_eq!(node, Node::Inferior(Vec::new()));
//     }

//     #[test]
//     fn test_new_superior_or_equal() {
//         // Test the creation of a new superior or equal node
//         let node = Node::new_superior_or_equal();
//         assert_eq!(node, Node::SuperiorOrEqual(Vec::new()));
//     }

//     #[test]
//     fn test_new_equal() {
//         // Test the creation of a new equal node
//         let node = Node::new_equal();
//         assert_eq!(node, Node::Equal(Vec::new()));
//     }

//     #[test]
//     fn test_new_if() {
//         // Test the creation of a new if node
//         let node = Node::new_if();
//         assert_eq!(
//             node,
//             Node::If(Vec::new(), None, OnceLock::new(), None, None)
//         );
//     }

//     #[test]
//     fn test_new_unary_plus() {
//         // Test the creation of a new unary plus node
//         let node = Node::new_unary_plus();
//         assert_eq!(node, Node::UnaryPlus(Vec::new()));
//     }

//     #[test]
//     fn test_new_unary_minus() {
//         // Test the creation of a new unary minus node
//         let node = Node::new_unary_minus();
//         assert_eq!(node, Node::UnaryMinus(Vec::new()));
//     }

//     #[test]
//     fn test_new_inferior_or_equal() {
//         // Test the creation of a new inferior or equal node
//         let node = Node::new_inferior_or_equal();
//         assert_eq!(node, Node::InferiorOrEqual(Vec::new()));
//     }

//     #[test]
//     fn test_new_not_equal() {
//         // Test the creation of a new not equal node
//         let node = Node::new_not_equal();
//         assert_eq!(node, Node::NotEqual(Vec::new()));
//     }

//     #[test]
//     fn test_new_true() {
//         // Test the creation of a new true node
//         let node = Node::new_true();
//         assert_eq!(node, Node::True);
//     }

//     #[test]
//     fn test_new_false() {
//         // Test the creation of a new false node
//         let node = Node::new_false();
//         assert_eq!(node, Node::False);
//     }

//     #[test]
//     fn test_new_pays() {
//         // Test the creation of a new pays node
//         let node = Node::new_pays();
//         assert_eq!(
//             node,
//             Node::Pays(Vec::new(), None, None, OnceLock::new(), OnceLock::new())
//         );
//     }

//     #[test]
//     fn test_new_spot_without_date() {
//         // Spot node defaults to None when no date is provided
//         let node = Node::new_spot(Currency::USD, Currency::EUR, None);
//         assert_eq!(
//             node,
//             Node::Spot(Currency::USD, Currency::EUR, None, OnceLock::new())
//         );
//     }

//     #[test]
//     fn test_new_spot_with_date() {
//         // Spot node can be created with an explicit date
//         let date = Date::new(2025, 6, 1);
//         let node = Node::new_spot(Currency::USD, Currency::EUR, Some(date));
//         assert_eq!(
//             node,
//             Node::Spot(Currency::USD, Currency::EUR, Some(date), OnceLock::new())
//         );
//     }

//     #[test]
//     fn test_new_df() {
//         let date = Date::new(2025, 6, 1);
//         let node = Node::new_df(date, Some("curve".to_string()));
//         assert_eq!(
//             node,
//             Node::Df(date, Some("curve".to_string()), OnceLock::new())
//         );
//     }

//     #[test]
//     fn test_new_rate_index() {
//         let start = Date::new(2024, 1, 1);
//         let end = Date::new(2024, 2, 1);
//         let node = Node::new_rate_index("0".to_string(), start, end);
//         assert_eq!(
//             node,
//             Node::RateIndex("0".to_string(), start, end, OnceLock::new())
//         );
//     }

//     #[test]
//     fn test_new_range() {
//         let node = Node::new_range();
//         assert_eq!(node, Node::Range(Vec::new()));
//     }

//     #[test]
//     fn test_new_list() {
//         let node = Node::new_list();
//         assert_eq!(node, Node::List(Vec::new()));
//     }

//     #[test]
//     fn test_new_append() {
//         let node = Node::new_append();
//         assert_eq!(node, Node::Append(Vec::new()));
//     }

//     #[test]
//     fn test_new_mean() {
//         let node = Node::new_mean();
//         assert_eq!(node, Node::Mean(Vec::new()));
//     }

//     #[test]
//     fn test_new_std() {
//         let node = Node::new_std();
//         assert_eq!(node, Node::Std(Vec::new()));
//     }

//     #[test]
//     fn test_new_index() {
//         let node = Node::new_index();
//         assert_eq!(node, Node::Index(Vec::new()));
//     }

//     #[test]
//     fn test_new_for_each() {
//         let iter = Box::new(Node::new_range());
//         let body: Vec<ExprTree> = Vec::new();
//         let node = Node::new_for_each("i".to_string(), iter.clone(), body.clone());
//         assert_eq!(
//             node,
//             Node::ForEach("i".to_string(), iter, body, OnceLock::new())
//         );
//     }

//     #[test]
//     fn test_add_child_to_base() {
//         // Test adding a child to a base node
//         let mut node = Node::new_base();
//         let child = Box::new(Node::new_add());
//         node.add_child(child.clone());
//         assert_eq!(node.children(), &vec![child]);
//     }

//     #[test]
//     fn test_add_child_to_add() {
//         // Test adding a child to an add node
//         let mut node = Node::new_add();
//         let child = Box::new(Node::new_subtract());
//         node.add_child(child.clone());
//         assert_eq!(node.children(), &vec![child]);
//     }

//     #[test]
//     #[should_panic(expected = "Cannot add child to spot node")]
//     fn test_add_child_to_spot() {
//         // Test adding a child to a spot node, which should panic
//         let mut node = Node::Spot(Currency::USD, Currency::AUD, None, OnceLock::new());
//         let child = Box::new(Node::new_add());
//         node.add_child(child);
//     }

//     #[test]
//     #[should_panic(expected = "Cannot add child to df node")]
//     fn test_add_child_to_df() {
//         let mut node = Node::new_df(Date::new(2025, 1, 1), None);
//         node.add_child(Box::new(Node::new_add()));
//     }

//     #[test]
//     #[should_panic(expected = "Cannot add child to rate index node")]
//     fn test_add_child_to_rate_index() {
//         let mut node = Node::new_rate_index(
//             "0".to_string(),
//             Date::new(2024, 1, 1),
//             Date::new(2024, 2, 1),
//         );
//         node.add_child(Box::new(Node::new_add()));
//     }

//     #[test]
//     #[should_panic(expected = "Cannot add child to true node")]
//     fn test_add_child_to_true() {
//         // Test adding a child to a true node, which should panic
//         let mut node = Node::True;
//         let child = Box::new(Node::new_add());
//         node.add_child(child);
//     }

//     #[test]
//     #[should_panic(expected = "Cannot add child to constant node")]
//     fn test_add_child_to_constant() {
//         // Test adding a child to a constant node, which should panic
//         let mut node = Node::Constant(NumericType::new(3.14));
//         let child = Box::new(Node::new_add());
//         node.add_child(child);
//     }

//     #[test]
//     fn test_children_of_base() {
//         // Test getting children of a base node
//         let mut node = Node::new_base();
//         let child = Box::new(Node::new_add());
//         node.add_child(child.clone());
//         assert_eq!(node.children(), &vec![child]);
//     }

//     #[test]
//     #[should_panic(expected = "Cannot get children from spot node")]
//     fn test_children_of_spot() {
//         // Test getting children of a spot node, which should panic
//         let node = Node::Spot(Currency::USD, Currency::AUD, None, OnceLock::new());
//         node.children();
//     }

//     #[test]
//     #[should_panic(expected = "Cannot get children from df node")]
//     fn test_children_of_df() {
//         let node = Node::new_df(Date::new(2025, 1, 1), None);
//         node.children();
//     }

//     #[test]
//     #[should_panic(expected = "Cannot get children from rate index node")]
//     fn test_children_of_rate_index() {
//         let node = Node::new_rate_index(
//             "0".to_string(),
//             Date::new(2024, 1, 1),
//             Date::new(2024, 2, 1),
//         );
//         node.children();
//     }

//     #[test]
//     #[should_panic(expected = "Cannot get children from true node")]
//     fn test_children_of_true() {
//         // Test getting children of a true node, which should panic
//         let node = Node::True;
//         node.children();
//     }

//     #[test]
//     #[should_panic(expected = "Cannot get children from constant node")]
//     fn test_children_of_constant() {
//         // Test getting children of a constant node, which should panic
//         let node = Node::Constant(NumericType::new(3.14));
//         node.children();
//     }
// }
