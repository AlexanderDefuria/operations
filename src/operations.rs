use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ptr::hash;

#[derive(Debug, Clone)]
pub enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Exponentiation,
    Collect,
    Value(f64),
    Variable(String),
}

impl Operation {
    pub fn equation_repr(&self) -> String {
        match self {
            Operation::Add => "+".to_string(),
            Operation::Subtract => "-".to_string(),
            Operation::Multiply => "*".to_string(),
            Operation::Divide => "/".to_string(),
            Operation::Exponentiation => "^".to_string(),
            Operation::Collect => "collect".to_string(),
            Operation::Value(a) => {
                if a == &0.0 {
                    format!("{:.1}", a)
                } else {
                    a.to_string()
                }
            }
            Operation::Variable(a) => a.to_string(),
        }
    }

    pub fn matches(&self, rs: &Operation) -> bool {
        match (self, rs) {
            (Operation::Add, Operation::Add) => true,
            (Operation::Subtract, Operation::Subtract) => true,
            (Operation::Multiply, Operation::Multiply) => true,
            (Operation::Divide, Operation::Divide) => true,
            (Operation::Exponentiation, Operation::Exponentiation) => true,
            (Operation::Collect, Operation::Collect) => true,
            (Operation::Value(_), Operation::Value(_)) => true,
            (Operation::Variable(_), Operation::Variable(_)) => true,
            _ => false,
        }
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::Value(a) => write!(f, "Operation::Value({:.1})", a),
            Operation::Variable(a) => write!(f, "Operation::Variable(\"{}\")", a),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl PartialEq for Operation {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Operation::Value(a), Operation::Value(b)) => a == b,
            (Operation::Variable(a), Operation::Variable(b)) => a == b,
            (Operation::Add, Operation::Add) => true,
            (Operation::Subtract, Operation::Subtract) => true,
            (Operation::Multiply, Operation::Multiply) => true,
            (Operation::Divide, Operation::Divide) => true,
            (Operation::Collect, Operation::Collect) => true,
            _ => false,
        }
    }
}

impl Eq for Operation {}

impl Hash for Operation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash(self, state);
    }
}
