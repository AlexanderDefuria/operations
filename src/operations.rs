use crate::math::{EquationMember, EquationRepr};
use crate::prelude::*;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Add;
use std::ptr::hash;
use std::rc::Rc;

#[derive(Clone)]
pub enum Operation {
    Multiply(Vec<Operation>),
    Negate(Option<Box<Operation>>),
    Divide(Option<Box<Operation>>, Option<Box<Operation>>),
    Sum(Vec<Operation>),
    Value(Rc<dyn EquationMember>),
    Text(String),
    Mapping(usize),
}


impl EquationMember for Operation {
    fn equation_repr(&self) -> String {
        match self {
            Multiply(list) => {
                let mut string = String::new();
                for (i, item) in list.iter().enumerate() {
                    string.push_str(&item.equation_repr());
                    if i != list.len() - 1 {
                        string.push_str(" * ");
                    }
                }
                string
            }
            Negate(a) => {
                match a {
                    Some(op) => match op.as_ref() {
                        Negate(Some(x)) => {
                            return format!("{}", x.equation_repr());
                        }
                        _ => {}
                    },
                    None => {}
                }

                format!("-{}", a.clone().unwrap().equation_repr())
            }
            Divide(Some(a), Some(b)) => {
                format!("{}/{}", a.equation_repr(), b.equation_repr())
            }
            Sum(vec) => {
                let mut string = String::new();
                for (i, item) in vec.iter().enumerate() {
                    string.push_str(&item.equation_repr());
                    if i != vec.len() - 1 {
                        string.push_str(" + ");
                    }
                }
                string
            }
            Value(a) => a.equation_repr(),
            Mapping(a) => a.equation_repr(),
            Text(a) => a.clone(),
            _ => {
                panic!("Not implemented");
            }
        }
    }

    fn value(&self) -> f64 {
        match self {
            Multiply(list) => {
                let mut product = 1.0;
                for item in list {
                    product *= item.value();
                }
                product
            }
            Negate(Some(a)) => -a.value(),
            Sum(vec) => {
                let mut sum = 0.0;
                for item in vec {
                    sum += item.value();
                }
                sum
            }
            Divide(Some(a), Some(b)) => a.value() / b.value(),
            Value(a) => a.value(),
            Mapping(a) => a.value(),
            Text(_) => 0.0,
            _ => {
                panic!("Not implemented");
            }
        }
    }

    /// Simplifies the operation, returning `Some(_)` new operation if possible.
    /// returning `None` if the operation cannot be simplified.
    fn simplify(&self) -> Option<Operation> {
        match self {
            Multiply(list) => {
                let mut coefficient: f64 = 1.0;
                let mut result: Vec<Operation> = Vec::new();
                list.iter().for_each(|x| match x {
                    Value(a) => coefficient *= a.value(),
                    Mapping(_) | Text(_) => result.push(x.clone()),
                    _ => {
                        if let Some(child_simplification) = x.simplify() {
                            coefficient *= child_simplification.value();
                        } else {
                            result.push(x.clone());
                        }
                    }
                });
                result.push(Value(Rc::new(coefficient)));
                if result.len() == 1 {
                    return Some(result[0].clone());
                }
                return Some(Multiply(result));
            }
            Negate(child) => {
                if let Some(child) = child {
                    match child.as_ref() {
                        Negate(second_child) => {
                            if let Some(second_child) = second_child {
                                return Some(*second_child.clone());
                            }
                        }
                        Value(a) => return Some(Value(Rc::new(-a.value()))),
                        _ => {
                            let result = child.simplify();
                            if let Some(result) = result {
                                if let Value(a) = result {
                                    return Some(Value(Rc::new(-a.value())));
                                }
                                if let Negate(Some(x)) = result {
                                    return Some(*x);
                                }
                                return Some(Negate(Some(Box::new(result))));
                            }
                        }
                    }
                }
            }
            Divide(numerator, divisor) => {
                if let Some(numerator) = numerator {
                    if let Some(divisor) = divisor {
                        let simplification: (Option<Operation>, Option<Operation>) =
                            (numerator.simplify(), divisor.simplify());
                        if let (Some(Value(a)), Some(Value(b))) =
                            (&simplification.0, &simplification.1)
                        {
                            return Some(Value(Rc::new(a.value() / b.value())));
                        }
                        if let (None, None) = simplification {
                            return None;
                        }
                        let a = simplification.0.unwrap_or_else(|| *numerator.clone());
                        let b = simplification.1.unwrap_or_else(|| *divisor.clone());
                        return Some(Divide(Some(Box::new(a)), Some(Box::new(b))));
                    }
                }
            }
            Sum(list) => {
                let mut total: f64 = 0.0;
                let mut result: Vec<Operation> = Vec::new();
                list.iter().for_each(|x| match x {
                    Value(a) => total += a.value(),
                    Mapping(_) | Text(_) => result.push(x.clone()),
                    _ => {
                        if let Some(child_simplification) = x.simplify() {
                            total += child_simplification.value();
                        } else {
                            result.push(x.clone());
                        }
                    }
                });
                result.push(Value(Rc::new(total)));
                if result.len() == 1 {
                    return Some(result[0].clone());
                }
                return Some(Sum(result));
            }
            Value(a) => return Some(Value(a.clone())),
            Mapping(_) => {}
            Text(_) => {}
        }

        None
    }

    fn latex_string(&self) -> String {
        match self {
            Multiply(list) => {
                let mut string = String::new();
                for (i, item) in list.iter().enumerate() {
                    string.push_str(&item.latex_string());
                    if i != list.len() - 1 {
                        string.push_str(" \\cdot ");
                    }
                }
                string
            }
            Negate(Some(a)) => {
                format!("-{}", a.latex_string())
            }
            Sum(vec) => {
                let mut string = String::new();
                string.push_str("{");
                for (i, item) in vec.iter().enumerate() {
                    string.push_str(&item.latex_string());
                    if i != vec.len() - 1 {
                        string.push_str(" + ");
                    }
                }
                string.push_str("}");
                string
            }
            Divide(Some(a), Some(b)) => {
                format!("\\frac{{{}}}{{{}}}", a.latex_string(), b.latex_string())
            }
            Value(a) => a.latex_string(),
            Mapping(a) => a.latex_string(),
            Text(a) => a.clone(),
            _ => "Not implemented".to_string(),
        }
    }
}

impl Operation {
    /// Checks if the operation matches the given operation.
    /// Text (Variable) and Value operations are considered to match each other.
    pub fn matches(&self, rs: &Operation) -> bool {
        match (self, rs) {
            (Sum(_), Sum(_)) => true,
            (Multiply(_), Multiply(_)) => true,
            (Negate(_), Negate(_)) => true,
            (Divide(_, _), Divide(_, _)) => true,
            (Mapping(_), Mapping(_)) => true,
            (Value(_) | Text(_) | Mapping(_), Value(_) | Text(_) | Mapping(_)) => true,
            _ => false,
        }
    }

    pub fn compare_entire_structure(&self, rs: &Operation) -> bool {
        self.compare_structure(rs, 255)
    }

    pub fn compare_structure(&self, rs: &Operation, level: usize) -> bool {
        if level == 0 {
            return true;
        }
        let next_level: usize = level - 1;

        match (self, rs) {
            (Sum(ls), Sum(rs)) | (Multiply(ls), Multiply(rs)) => {
                if ls.len() != rs.len() {
                    return false;
                }
                for (l, r) in ls.iter().zip(rs.iter()) {
                    if !l.compare_structure(r, next_level) {
                        return false;
                    }
                }
                true
            }
            (Negate(ls), Negate(rs)) => match (ls, rs) {
                (Some(ls), Some(rs)) => ls.compare_structure(rs, next_level),
                _ => false,
            },
            (Negate(ls), _) => match ls {
                Some(ls) => ls.compare_structure(rs, next_level),
                _ => false,
            },
            (_, Negate(rs)) => match rs {
                Some(rs) => self.compare_structure(rs, next_level),
                _ => false,
            },
            (Divide(lsn, lsd), Divide(rsn, rsd)) => {
                let denominator: bool = match (lsd, rsd) {
                    (Some(ls), Some(rs)) => ls.compare_structure(rs, next_level),
                    _ => false,
                };
                let numerator_match: bool = match (lsn, rsn) {
                    (Some(ls), Some(rs)) => ls.compare_structure(rs, next_level),
                    _ => false,
                };
                denominator && numerator_match
            }
            (a, b) => a.matches(b),
        }
    }
}

impl Debug for Operation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.equation_repr())
    }
}

impl PartialEq for Operation {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value(a), Value(b)) => a.value() == b.value(),
            (Text(a), Text(b)) => a == b,
            (Multiply(a), Multiply(b)) => a.iter().all(|x| b.contains(x)) && b.len() == a.len(),
            (Negate(a), Negate(b)) => a == b,
            (Divide(a, b), Divide(c, d)) => a == c && b == d,
            (Sum(a), Sum(b)) => a.iter().all(|x| b.contains(x)) && b.len() == a.len(),
            (Mapping(a), Mapping(b)) => a == b,
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

impl Add for Operation {
    type Output = Operation;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Sum(mut a), Sum(mut b)) => {
                a.append(&mut b);
                Sum(a)
            }
            (Sum(mut a), b) => {
                a.push(b);
                Sum(a)
            }
            (a, Sum(mut b)) => {
                b.insert(0, a);
                Sum(b)
            }
            (Value(a), Value(b)) => Value(Rc::new(a.value() + b.value())),
            (a, b) => Sum(vec![a, b]),
        }
    }
}

impl num_traits::Zero for Operation {
    fn zero() -> Self {
        Value(Rc::new(0.0))
    }

    fn is_zero(&self) -> bool {
        match self {
            Value(a) => a.is_zero(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::math::EquationMember;
    use crate::prelude::*;
    use std::rc::Rc;

    #[test]
    fn test_multiplication_simplification() {
        let a: Operation = Multiply(vec![Value(Rc::new(2.0)), Value(Rc::new(3.0))]);
        assert_eq!(a.simplify(), Some(Value(Rc::new(6.0))));

        let a: Operation = Multiply(vec![
            Value(Rc::new(2.0)),
            Value(Rc::new(3.0)),
            Value(Rc::new(4.0)),
        ]);
        assert_eq!(a.simplify(), Some(Value(Rc::new(24.0))));

        let a: Operation = Multiply(vec![
            Value(Rc::new(2.0)),
            Value(Rc::new(3.0)),
            Text("x".to_string()),
        ]);
        assert_eq!(
            a.simplify(),
            Some(Multiply(vec![Value(Rc::new(6.0)), Text("x".to_string())]))
        );

        let a: Operation = Multiply(vec![
            Value(Rc::new(3.0)),
            Value(Rc::new(2.0)),
            Text("x".to_string()),
            Text("y".to_string()),
        ]);
        assert_eq!(
            a.simplify(),
            Some(Multiply(vec![
                Value(Rc::new(6.0)),
                Text("x".to_string()),
                Text("y".to_string())
            ]))
        );
    }

    #[test]
    fn test_negation_simplification() {
        let a: Operation = Negate(Some(Box::new(Value(Rc::new(2.0)))));
        assert_eq!(a.simplify(), Some(Value(Rc::new(-2.0))));

        let a: Operation = Negate(Some(Box::new(Negate(Some(Box::new(Value(Rc::new(2.0))))))));
        assert_eq!(a.simplify(), Some(Value(Rc::new(2.0))));

        let a: Operation = Negate(Some(Box::new(Multiply(vec![
            Value(Rc::new(2.0)),
            Value(Rc::new(3.0)),
        ]))));
        assert_eq!(a.simplify(), Some(Value(Rc::new(-6.0))));

        let a: Operation = Negate(Some(Box::new(Multiply(vec![Value(Rc::new(2.0))]))));
        assert_eq!(a.simplify(), Some(Value(Rc::new(-2.0))));
    }

    #[test]
    fn test_division_simplification() {
        let a: Operation = Divide(
            Some(Box::new(Value(Rc::new(2.0)))),
            Some(Box::new(Value(Rc::new(3.0)))),
        );
        assert_eq!(a.simplify(), Some(Value(Rc::new(2.0 / 3.0))));

        let a: Operation = Divide(
            Some(Box::new(Text("x".to_string()))),
            Some(Box::new(Value(Rc::new(3.0)))),
        );
        assert_eq!(
            a.simplify(),
            Some(Divide(
                Some(Box::new(Text("x".to_string()))),
                Some(Box::new(Value(Rc::new(3.0))))
            ))
        );

        let a: Operation = Divide(
            Some(Box::new(Value(Rc::new(2.0)))),
            Some(Box::new(Text("x".to_string()))),
        );
        assert_eq!(
            a.simplify(),
            Some(Divide(
                Some(Box::new(Value(Rc::new(2.0)))),
                Some(Box::new(Text("x".to_string())))
            ))
        );

        // There is no simplification for x/y thus return None
        let a: Operation = Divide(
            Some(Box::new(Text("x".to_string()))),
            Some(Box::new(Text("y".to_string()))),
        );
        assert_eq!(a.simplify(), None);

        let a: Operation = Divide(
            Some(Box::new(Negate(Some(Box::new(Multiply(vec![
                Value(Rc::new(2.0)),
                Value(Rc::new(3.0)),
            ])))))),
            Some(Box::new(Text("x".to_string()))),
        );
        assert_eq!(
            a.simplify(),
            Some(Divide(
                Some(Box::new(Value(Rc::new(-6.0)))),
                Some(Box::new(Text("x".to_string())))
            ))
        );

        let a: Operation = Divide(
            Some(Box::new(Negate(Some(Box::new(Multiply(vec![
                Value(Rc::new(2.0)),
                Value(Rc::new(3.0)),
            ])))))),
            Some(Box::new(Negate(Some(Box::new(Value(Rc::new(2.0))))))),
        );
        assert_eq!(a.simplify(), Some(Value(Rc::new(3.0))));

        let a: Operation = Divide(
            Some(Box::new(Negate(Some(Box::new(Multiply(vec![
                Text("x".to_string()),
                Value(Rc::new(2.0)),
            ])))))),
            Some(Box::new(Negate(Some(Box::new(Value(Rc::new(2.0))))))),
        );
        assert_eq!(
            a.simplify(),
            Some(Divide(
                Some(Box::new(Negate(Some(Box::new(Multiply(vec![
                    Text("x".to_string()),
                    Value(Rc::new(2.0)),
                ])))))),
                Some(Box::new(Value(Rc::new(-2.0))))
            ),)
        );
    }

    #[test]
    fn test_summation_simplification() {
        let a: Operation = Sum(vec![Value(Rc::new(2.0)), Value(Rc::new(3.0))]);
        assert_eq!(a.simplify(), Some(Value(Rc::new(5.0))));

        let a: Operation = Sum(vec![
            Value(Rc::new(2.0)),
            Value(Rc::new(3.0)),
            Value(Rc::new(4.0)),
        ]);
        assert_eq!(a.simplify(), Some(Value(Rc::new(9.0))));

        let a: Operation = Sum(vec![
            Value(Rc::new(2.0)),
            Value(Rc::new(3.0)),
            Text("x".to_string()),
        ]);
        assert_eq!(
            a.simplify(),
            Some(Sum(vec![Value(Rc::new(5.0)), Text("x".to_string())]))
        );

        let a: Operation = Sum(vec![
            Value(Rc::new(2.0)),
            Value(Rc::new(3.0)),
            Text("x".to_string()),
            Text("y".to_string()),
        ]);
        assert_eq!(
            a.simplify(),
            Some(Sum(vec![
                Value(Rc::new(5.0)),
                Text("x".to_string()),
                Text("y".to_string())
            ]))
        );

        let a: Operation = Sum(vec![
            Value(Rc::new(2.0)),
            Value(Rc::new(3.0)),
            Text("x".to_string()),
            Text("y".to_string()),
            Value(Rc::new(4.0)),
        ]);
        assert_eq!(
            a.simplify(),
            Some(Sum(vec![
                Value(Rc::new(9.0)),
                Text("x".to_string()),
                Text("y".to_string())
            ]))
        );
    }
}
