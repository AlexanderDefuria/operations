use crate::algorithms::{binary_tree_algorithm, shunting_yard_algorithm};
use crate::operations::Operation;
use crate::prelude::{Add, Divide, Multiply, Subtract, Value, Variable};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use Operation::{Collect, Exponentiation};

#[derive(Debug, Clone)]
pub struct Equation {
    pub root: Operation,
    pub left: Option<Box<Equation>>,
    pub right: Option<Box<Equation>>,
}

pub trait Malleable {
    fn malleable(&self) -> bool {
        true
    }
    fn simplify(&mut self) -> Self;
}

impl Malleable for Vec<Equation> {
    fn simplify(&mut self) -> Vec<Equation> {
        self.iter_mut().map(|x| x.simplify()).collect()
    }
}

impl Equation {
    pub fn new(root: Operation) -> Equation {
        Equation {
            root,
            left: None,
            right: None,
        }
    }

    pub fn leaf(&self) -> bool {
        self.left.is_none() || self.right.is_none()
    }

    pub fn equation_repr(&self) -> String {
        let mut out = String::new();
        if let Some(left) = &self.left {
            out.push_str(format!("{}", left.equation_repr()).as_str());
            if !left.leaf() {
                out = "(".to_string() + &out + ")";
            }
        }
        out.push_str(format!("{}", self.root.equation_repr()).as_str());
        if let Some(right) = &self.right {
            out.push_str(format!("{}", right.equation_repr()).as_str());
            if !right.leaf() {
                out = "(".to_string() + &out + ")";
            }
        }
        out
    }

    pub fn get_variables(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        match &self.root {
            Value(x) => out.push(x.to_string()),
            Variable(text) => out.push(text.clone()),
            _ => {}
        }
        if let Some(left) = &self.left {
            out.append(&mut left.get_variables());
        }
        if let Some(right) = &self.right {
            out.append(&mut right.get_variables());
        }

        out
    }

    pub fn replace_variable(&mut self, original: String, new: String) {
        if let Variable(text) = &mut self.root {
            if text.to_string() == original {
                if let Ok(value) = f64::from_str(&*new) {
                    *self = Equation::new(Value(value));
                    return;
                }
                *text = new.clone();
            }
        }

        if let Some(left) = &mut self.left {
            left.replace_variable(original.clone(), new.clone());
        }
        if let Some(right) = &mut self.right {
            right.replace_variable(original.clone(), new.clone());
        }
    }

    pub(crate) fn set_operation(&mut self, root: Operation) {
        match root {
            Value(_) => {}
            Variable(_) => {}
            _ => {
                panic!("Cannot set with non-leaf root")
            }
        }
        self.root = root;
        self.left = None;
        self.right = None;
    }

    /// Set self equal to the input.
    pub(crate) fn set_equation(&mut self, equation: Box<Equation>) {
        self.root = equation.root;
        self.left = equation.left;
        self.right = equation.right;
    }

    pub fn collect_summations(&self) -> Vec<Equation> {
        let mut out: Vec<Equation> = Vec::new();

        fn negate(input: Equation) -> Equation {
            Equation {
                root: Multiply,
                left: Some(Box::new(Equation::new(Value(-1.0)))),
                right: Some(Box::new(input)),
            }
        }

        if self.is_summation() {
            if self.root.matches(&Subtract) {
                self.right
                    .as_ref()
                    .unwrap()
                    .collect_summations()
                    .iter()
                    .for_each(|x| {
                        out.push(negate(x.clone()));
                    });
            } else {
                if let Some(right) = &self.right {
                    out.append(&mut right.collect_summations());
                }
            }
            if let Some(left) = &self.left {
                out.append(&mut left.collect_summations());
            }
        } else {
            out.push(self.clone());
        }

        out
    }

    #[allow(dead_code)]
    pub fn declaration(&self) -> String {
        let mut out = String::new();
        out.push_str("Equation {");

        out.push_str("root: ");
        match &self.root {
            Variable(text) => {
                out.push_str(format!("Variable(\"{text}\".parse().unwrap())").as_str())
            }
            _ => out.push_str(format!("{:?}", self.root).as_str()),
        }
        out.push_str(",");

        out.push_str("left: ");
        match &self.left {
            Some(left) => {
                out.push_str("Some(Box::new(");
                out.push_str(format!("{}", left.declaration()).as_str());
                out.push_str(")),");
            }
            None => {
                out.push_str("None,");
            }
        }

        out.push_str("right: ");
        match &self.right {
            Some(right) => {
                out.push_str("Some(Box::new(");
                out.push_str(format!("{}", right.declaration()).as_str());
                out.push_str(")),");
            }
            None => {
                out.push_str("None,");
            }
        }

        out.push_str("}");
        out
    }

    pub fn compare_structure(&self, rs: Equation) -> bool {

        fn match_structure(ls: &Operation, rs: &Operation) -> bool {
            match (ls, rs) {
                (Add, Add) => true,
                (Subtract, Subtract) => true,
                (Multiply, Multiply) => true,
                (Divide, Divide) => true,
                (Exponentiation, Exponentiation) => true,
                (Collect, Collect) => true,
                (Value(_), Value(_)) => true,
                (Variable(_), Value(_)) => true,
                (Value(_), Variable(_)) => true,
                (Variable(_), Variable(_)) => true,
                _ => false,
            }
        }

        if !self.leaf() && !rs.leaf() {
            if match_structure(&self.root, &rs.root) {
                return self.left
                    .clone()
                    .unwrap()
                    .compare_structure(*rs.left.clone().unwrap())
                    && self
                    .right
                    .clone()
                    .unwrap()
                    .compare_structure(*rs.right.clone().unwrap());
            }
        } else if self.leaf() && rs.leaf() {
            return match_structure(&self.root, &rs.root);
        }

        false


    }

    pub fn is_summation(&self) -> bool {
        return match &self.root {
            Add | Subtract => self.left.is_some() && self.right.is_some(),
            _ => false,
        };
    }
}

impl Malleable for Equation {
    fn simplify(&mut self) -> Equation {
        let original: String = self.equation_repr();
        match &mut self.root {
            Add | Subtract => match (&self.left, &self.right) {
                (Some(left), Some(right)) => match (&left.root, &right.root) {
                    (Value(a), Value(b)) => match self.root {
                        Add => self.set_operation(Value(a + b)),
                        Subtract => self.set_operation(Value(a - b)),
                        _ => {}
                    },
                    (_, Value(0.0)) => self.set_equation(self.left.clone().unwrap()),
                    (Value(0.0), _) => match self.root {
                        Add => self.set_equation(self.right.clone().unwrap()),
                        _ => {}
                    },
                    _ => {}
                },
                (Some(left), None) => self.set_equation(left.clone()),
                (None, Some(right)) => match self.root {
                    Subtract => {
                        self.root = Multiply;
                        self.left = Some(Box::new(Equation::new(Value(-1.0))));
                    }
                    _ => self.set_equation(right.clone()),
                },
                (_, _) => (),
            },
            Multiply => match (&self.left, &self.right) {
                (Some(left), Some(right)) => match (&left.root, &right.root) {
                    (Value(a), Value(b)) => self.set_operation(Value(a * b)),
                    (_, Value(0.0)) => self.set_equation(Box::new(Equation::new(Value(0.0)))),
                    (Value(0.0), _) => self.set_equation(Box::new(Equation::new(Value(0.0)))),
                    (Value(1.0), _) => self.set_equation(right.clone()),
                    (_, Value(1.0)) => self.set_equation(left.clone()),
                    _ => {}
                },
                (Some(_), None) => self.set_operation(Value(0.0)),
                (None, Some(_)) => self.set_operation(Value(0.0)),
                (_, _) => panic!("Multiply operation must have at least one child"),
            },
            Divide => match (&self.left, &self.right) {
                (Some(left), Some(right)) => match (&left.root, &right.root) {
                    (Value(a), Value(b)) => self.set_operation(Value(a / b)),
                    (_, Value(1.0)) => self.set_equation(left.clone()),
                    _ => {}
                },
                (None, Some(_)) => self.set_operation(Value(0.0)),
                (_, _) => panic!(
                    "Divide operation must have at least 1 child and a divisor (right child)"
                ),
            },
            _ => {}
        }
        if let Some(left) = &mut self.left {
            left.simplify();
        }
        if let Some(right) = &mut self.right {
            right.simplify();
        }

        if original != self.equation_repr() {
            self.simplify();
        }

        self.clone()
    }
}

impl Display for Equation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.root {
            Value(a) => write!(f, "{}", a),
            Variable(a) => write!(f, "{}", a),
            _ => write!(f, "{:?}", self.root),
        }
    }
}

impl PartialEq for Equation {
    fn eq(&self, other: &Self) -> bool {
        self.root == other.root && self.left == other.left && self.right == other.right
    }
}

impl From<String> for Equation {
    fn from(value: String) -> Self {
        let postfix: Vec<String> = shunting_yard_algorithm(value);
        binary_tree_algorithm(postfix)
    }
}

impl FromStr for Equation {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Equation::from(s.to_string()))
    }
}

impl Into<String> for Equation {
    fn into(self) -> String {
        self.equation_repr()
    }
}

impl Eq for Equation {}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn test_eq() {
        let a: Equation = Equation::from("(a+b)/c".to_string());
        let b: Equation = Equation::from("(a+b)/c".to_string());
        assert_eq!(a, b);
        let b: Equation = Equation::from("(a+b)/d".to_string());
        assert_ne!(a, b);
    }

    #[test]
    fn test_strings() {
        let a: Equation = Equation::from("(a+b)/c".to_string());
        assert_eq!(Into::<String>::into(a), "(a+b)/c".to_string());
    }

    #[test]
    fn test_get_variables() {
        let a: Equation = Equation::from("(a+b)/c".to_string());
        let mut b: Vec<String> = Vec::new();
        b.push("a".to_string());
        b.push("b".to_string());
        b.push("c".to_string());
        for i in 0..b.len() {
            assert_eq!(a.get_variables()[i], b[i]);
        }
    }

    #[test]
    fn test_replace_variable() {
        let mut a: Equation = Equation::from("(a+b)/c".to_string());
        a.replace_variable("a".to_string(), "d".to_string());
        assert_eq!(a, Equation::from("(d+b)/c".to_string()));
    }

    #[test]
    fn test_compare_structure() {
        let a: Equation = Equation::from("(a+b)/c".to_string());
        let b: Equation = Equation::from("(a+b)/c".to_string());
        assert!(a.compare_structure(b));
        let b: Equation = Equation::from("(a+b)/d".to_string());
        assert!(a.compare_structure(b));
    }

    #[test]
    fn test_basic_simplification() {
        let mut a: Equation = Equation::from("(a+0)/c".to_string());
        a.simplify();
        assert_eq!(a, Equation::from("a/c".to_string()));

        let mut a: Equation = Equation::from("(a-0)/c".to_string());
        a.simplify();
        assert_eq!(a, Equation::from("a/c".to_string()));

        let mut a: Equation = Equation::from("(0+1)/c".to_string());
        a.simplify();
        assert_eq!(a, Equation::from("1/c".to_string()));

        let mut a: Equation = Equation::from("(0-1)/c".to_string());
        a.simplify();
        assert_eq!(a, Equation::from("{-1}/c".to_string()));

        let mut a: Equation = Equation {
            root: Add,
            left: Some(Box::new(Equation {
                root: Divide,
                left: Some(Box::new(Equation {
                    root: Variable("{v_1}".parse().unwrap()),
                    left: None,
                    right: None,
                })),
                right: Some(Box::new(Equation {
                    root: Value(2.0),
                    left: None,
                    right: None,
                })),
            })),
            right: Some(Box::new(Equation {
                root: Value(0.0),
                left: None,
                right: None,
            })),
        };
        a.simplify();
        assert_eq!(a, Equation::from("{v_1}/2".to_string()));

        let mut a: Equation = Equation::from("0*{v_1}".to_string());
        a.simplify();
        assert_eq!(a, Equation::from("0".to_string()));

        let mut a: Equation = Equation {
            root: Subtract,
            left: None,
            right: Some(Box::from(Equation {
                root: Divide,
                left: Some(Box::from(Equation {
                    root: Multiply,
                    left: Some(Box::from(Equation {
                        root: Variable("{v_1}".parse().unwrap()),
                        left: None,
                        right: None,
                    })),
                    right: Some(Box::from(Equation {
                        root: Value(0.0),
                        left: None,
                        right: None,
                    })),
                })),
                right: Some(Box::from(Equation {
                    root: Value(2.0),
                    left: None,
                    right: None,
                })),
            })),
        };
        a.simplify();
        assert_eq!(a, Equation::from("0".to_string()));

        let mut a: Equation = Equation::from("-(a)".to_string());
        a.simplify();
        assert_eq!(a, Equation::from("{-1}*a".to_string()));

        let mut a: Equation = Equation::from("-(({v_1}-0)/2)".to_string());
        a.simplify();
        assert_eq!(a, Equation::from("{-1}*({v_1}/{2.0}) ".to_string()));
    }
}
