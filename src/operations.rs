use crate::math::EquationMember;
use crate::prelude::*;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::{Add, Index};
use std::ptr::hash;
use std::rc::Rc;

#[derive(Clone)]
pub enum Operation {
    Multiply(Vec<Operation>),
    Negate(Option<Box<Operation>>),
    Divide(Option<Box<Operation>>, Option<Box<Operation>>),
    Sum(Vec<Operation>),
    Value(f64),
    Text(String),
    Mapping(usize),
    Equal(Option<Box<Operation>>, Option<Box<Operation>>),
    Variable(Rc<dyn EquationMember>),
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
                let mut numerator = a.equation_repr();
                let mut denominator = b.equation_repr();
                match *a.clone() {
                    Multiply(a)| Sum(a) => {
                        if a.len() > 1 {
                            numerator = "{".to_owned() + numerator.as_str() + "}";
                        }
                    }
                    _ => {}
                }
                match *b.clone() {
                    Multiply(a)| Sum(a) => {
                        if a.len() > 1 {
                            denominator = "{".to_owned() + denominator.as_str() + "}";
                        }
                    }
                    _ => {}
                }
                format!("{}/{}", numerator, denominator)
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
            Equal(Some(a), Some(b)) => {
                format!("{} = {}", a.equation_repr(), b.equation_repr())
            }
            Variable(a) => a.equation_repr(),
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
            Mapping(_) | Text(_) => 1.0,
            Variable(a) => a.value(),
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
                result.push(Value(coefficient));
                if result.len() == 1 {
                    return Some(result[0].clone());
                }
                return Some(Multiply(result));
            }
            Sum(list) => {
                let mut total: f64 = 0.0;
                let mut result: Vec<Operation> = Vec::new();
                list.iter().for_each(|x| match x {
                    Value(a) => total += a.value(),
                    Mapping(_) | Text(_) | Variable(_) => result.push(x.clone()),
                    Sum(vec) => {
                        result.extend(vec.iter().cloned());
                    }
                    _ => {
                        if let Some(child_simplification) = x.simplify() {
                            total += child_simplification.value();
                        } else {
                            result.push(x.clone());
                        }
                    }
                });
                if total != 0.0 {
                    result.push(Value(total));
                }
                if result.len() == 1 {
                    return Some(result[0].clone());
                }
                return Some(Sum(result));
            }
            Negate(Some(child)) => match child.as_ref() {
                Negate(second_child) => {
                    if let Some(second_child) = second_child {
                        return Some(*second_child.clone());
                    }
                }
                Value(a) => return Some(Value(-a.value())),
                Sum(vec) => {
                    let mut result: Vec<Operation> = Vec::new();
                    for item in vec {
                        result.push(Negate(Some(Box::new(item.clone()))));
                    }
                    return Some(Sum(result));
                }
                _ => {
                    let result = child.simplify();
                    if let Some(result) = result {
                        if let Value(a) = result {
                            return Some(Value(-a.value()));
                        }
                        if let Negate(Some(x)) = result {
                            return Some(*x);
                        }
                        return Some(Negate(Some(Box::new(result))));
                    }
                }
            },
            Divide(Some(numerator), Some(divisor)) => {
                let simplification: (Option<Operation>, Option<Operation>) =
                    (numerator.simplify(), divisor.simplify());
                if let (Some(Value(a)), Some(Value(b))) = (&simplification.0, &simplification.1) {
                    return Some(Value(a.value() / b.value()));
                }
                if let (None, None) = simplification {
                    return None;
                }
                let a = simplification.0.unwrap_or_else(|| *numerator.clone());
                let b = simplification.1.unwrap_or_else(|| *divisor.clone());
                return Some(Divide(Some(Box::new(a)), Some(Box::new(b))));
            }
            Equal(Some(ls), Some(rs)) => {
                let simplification: (Option<Operation>, Option<Operation>) =
                    (ls.simplify(), rs.simplify());
                if let (None, None) = simplification {
                    return None;
                }
                let a = simplification.0.unwrap_or_else(|| *ls.clone());
                let b = simplification.1.unwrap_or_else(|| *rs.clone());
                return Some(Equal(Some(Box::new(a)), Some(Box::new(b))));
            }
            Value(_) => return Some(self.clone()),
            _ => {}
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
            Negate(Some(a)) => format!("-{{{}}}", a.latex_string()),
            Sum(vec) => {
                let mut string = String::from("{");
                for (i, item) in vec.iter().enumerate() {
                    if let Sum(_) | Multiply(_) = item {
                        string.push_str("{");
                    }
                    if let Negate(Some(a)) = item {
                        if i != 0 {
                            string.push_str(&a.latex_string());
                        } else {
                            string.push_str(&item.latex_string());
                        }
                    } else {
                        string.push_str(&item.latex_string());
                    }
                    if i != vec.len() - 1 {
                        if let Some(Negate(_)) = vec.get(i + 1) {
                            string.push_str(" - ");
                        } else {
                            string.push_str(" + ");
                        }
                    }
                    if let Sum(_) | Multiply(_) = item {
                        string.push_str("}");
                    }
                }
                string.push_str("}");
                string
            }
            Divide(Some(a), Some(b)) => {
                format!("\\frac{{{}}}{{{}}}", a.latex_string(), b.latex_string())
            }
            Equal(Some(a), Some(b)) => format!("{} = {}", a.latex_string(), b.latex_string()),
            Value(a) => a.latex_string(),
            Mapping(a) => a.latex_string(),
            Variable(a) => a.latex_string(),
            Text(a) => {
                format!("${}$", a)
            }
            _ => "$Not implemented$".to_string(),
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
            (Equal(_, _), Equal(_, _)) => true,
            _ => false,
        }
    }

    pub fn get_variables(&self) -> Vec<Operation> {
        let mut prelim: Vec<Operation> = Vec::new();
        match self {
            Multiply(list) => {
                for item in list {
                    prelim.extend(item.get_variables());
                }
            }
            Sum(list) => {
                for item in list {
                    prelim.extend(item.get_variables());
                }
            }
            Negate(Some(a)) => {
                prelim.extend(a.get_variables());
            }
            Divide(Some(a), Some(b)) => {
                prelim.extend(a.get_variables());
                prelim.extend(b.get_variables());
            }
            Equal(Some(a), Some(b)) => {
                prelim.extend(a.get_variables());
                prelim.extend(b.get_variables());
            }
            Variable(a) => {
                prelim.push(Variable(a.clone()));
            }
            _ => {}
        }

        // TODO: This function is not complete, its very inefficient.
        let mut out: Vec<Operation> = Vec::new();
        'outer: for item in prelim {
            for x in &out {
                if x.latex_string() == item.latex_string() {
                    continue 'outer;
                }
            }
            out.push(item);
        }
        out
    }

    pub fn apply_variables(&mut self) -> &mut Self {
        match self {
            Sum(vec) => {
                for item in vec {
                    item.apply_variables();
                }
            }
            Multiply(vec) => {
                for item in vec {
                    item.apply_variables();
                }
            }
            Negate(Some(a)) => {
                a.apply_variables();
            }
            Divide(Some(a), Some(b)) => {
                a.apply_variables();
                b.apply_variables();
            }
            Equal(Some(a), Some(b)) => {
                a.apply_variables();
                b.apply_variables();
            }
            Variable(a) => {
                let value: f64 = a.value();
                if value.is_finite() {
                    *self = Value(value);
                }
            }
            _ => {}
        }

        self
    }

    pub fn contains_variable(&self, rs: Operation) -> bool {
        match self {
            Multiply(list) | Sum(list) => list.iter().any(|x| x.contains_variable(rs.clone())),
            Negate(Some(a)) => a.contains_variable(rs),
            Divide(Some(a), Some(b)) | Equal(Some(a), Some(b)) => {
                a.contains_variable(rs.clone()) || b.contains_variable(rs)
            }
            _ => self.latex_string() == rs.latex_string(),
        }
    }

    /// Extracts the coefficient of an operation.
    ///
    /// NOTE The actual value is most certainly different than this result.
    pub fn get_coefficient(&self) -> Option<f64> {
        match self {
            Value(a) => Some(a.value()),
            Negate(Some(a)) => {
                if let Some(value) = a.get_coefficient() {
                    Some(-value)
                } else {
                    None
                }
            }
            Multiply(list) => {
                let mut coefficient: f64 = 1.0;
                for item in list {
                    if let Value(a) = item {
                        coefficient *= a.value();
                    }
                }
                Some(coefficient)
            }
            Divide(Some(a), Some(b)) => {
                if a.value().is_finite() {
                    Some(a.value() / b.value())
                } else {
                    match **a {
                        Negate(_) => Some(-1.0 / b.value()),
                        _ => Some(1.0 / b.value()),
                    }
                }
            }
            _ => None,
        }
    }

    pub fn print_operation_type(&self) -> &str {
        match self {
            Multiply(_) => "Multiply",
            Negate(_) => "Negate",
            Sum(_) => "Sum",
            Divide(_, _) => "Divide",
            Equal(_, _) => "Equal",
            Value(_) => "Value",
            Mapping(_) => "Mapping",
            Text(_) => "Text",
            Variable(_) => "Variable",
        }
    }

    pub fn compare_structure(&self, rs: &Operation) -> bool {
        match (self, rs) {
            (Sum(ls), Sum(rs)) | (Multiply(ls), Multiply(rs)) => {
                if ls.len() != rs.len() {
                    return false;
                }
                for (l, r) in ls.iter().zip(rs.iter()) {
                    if !l.compare_structure(r) {
                        return false;
                    }
                }
                true
            }
            (Negate(Some(ls)), Negate(Some(rs))) => ls.compare_structure(rs),
            (Negate(Some(ls)), _) => ls.compare_structure(rs),
            (_, Negate(Some(rs))) => rs.compare_structure(self),
            (Divide(Some(lsn), Some(lsd)), Divide(Some(rsn), Some(rsd))) => {
                let denominator: bool = lsd.compare_structure(rsd);
                let numerator_match: bool = lsn.compare_structure(rsn);
                denominator && numerator_match
            }
            (_, Mapping(_)) | (Mapping(_), _) => true,
            (a, b) => a.matches(b),
        }
    }

    pub fn cleanup(&mut self) {
        match self {
            Negate(Some(a)) => match *a.clone() {
                Negate(Some(b)) => {
                    *self = *b.clone();
                }
                _ => a.cleanup()
            },
            Sum(list) => list.iter_mut().for_each(|x| x.cleanup()),
            _ => {}
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
            (Value(a), Value(b)) => Value(a.value() + b.value()),
            (a, b) => Sum(vec![a, b]),
        }
    }
}

impl Index<usize> for Operation {
    type Output = Operation;

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            Multiply(a) => a.get(index).unwrap(),
            Sum(a) => a.get(index).unwrap(),
            _ => panic!("Cannot Index This Operation"),
        }
    }
}

impl num_traits::Zero for Operation {
    fn zero() -> Self {
        Value(0.0)
    }

    fn is_zero(&self) -> bool {
        match self {
            Variable(a) => a.is_zero(),
            Value(a) => *a == 0.0,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::math::EquationMember;
    use crate::prelude::*;

    #[test]
    fn test_multiplication_simplification() {
        let a: Operation = Multiply(vec![Value(2.0), Value(3.0)]);
        assert_eq!(a.simplify(), Some(Value(6.0)));

        let a: Operation = Multiply(vec![Value(2.0), Value(3.0), Value(4.0)]);
        assert_eq!(a.simplify(), Some(Value(24.0)));

        let a: Operation = Multiply(vec![Value(2.0), Value(3.0), Text("x".to_string())]);
        assert_eq!(
            a.simplify(),
            Some(Multiply(vec![Value(6.0), Text("x".to_string())]))
        );

        let a: Operation = Multiply(vec![
            Value(3.0),
            Value(2.0),
            Text("x".to_string()),
            Text("y".to_string()),
        ]);
        assert_eq!(
            a.simplify(),
            Some(Multiply(vec![
                Value(6.0),
                Text("x".to_string()),
                Text("y".to_string())
            ]))
        );
    }

    #[test]
    fn test_negation_simplification() {
        let a: Operation = Negate(Some(Box::new(Value(2.0))));
        assert_eq!(a.simplify(), Some(Value(-2.0)));

        let a: Operation = Negate(Some(Box::new(Negate(Some(Box::new(Value(2.0)))))));
        assert_eq!(a.simplify(), Some(Value(2.0)));

        let a: Operation = Negate(Some(Box::new(Multiply(vec![Value(2.0), Value(3.0)]))));
        assert_eq!(a.simplify(), Some(Value(-6.0)));

        let a: Operation = Negate(Some(Box::new(Multiply(vec![Value(2.0)]))));
        assert_eq!(a.simplify(), Some(Value(-2.0)));
    }

    #[test]
    fn test_division_simplification() {
        let a: Operation = Divide(Some(Box::new(Value(2.0))), Some(Box::new(Value(3.0))));
        assert_eq!(a.simplify(), Some(Value(2.0 / 3.0)));

        let a: Operation = Divide(
            Some(Box::new(Text("x".to_string()))),
            Some(Box::new(Value(3.0))),
        );
        assert_eq!(
            a.simplify(),
            Some(Divide(
                Some(Box::new(Text("x".to_string()))),
                Some(Box::new(Value(3.0)))
            ))
        );

        let a: Operation = Divide(
            Some(Box::new(Value(2.0))),
            Some(Box::new(Text("x".to_string()))),
        );
        assert_eq!(
            a.simplify(),
            Some(Divide(
                Some(Box::new(Value(2.0))),
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
                Value(2.0),
                Value(3.0),
            ])))))),
            Some(Box::new(Text("x".to_string()))),
        );
        assert_eq!(
            a.simplify(),
            Some(Divide(
                Some(Box::new(Value(-6.0))),
                Some(Box::new(Text("x".to_string())))
            ))
        );

        let a: Operation = Divide(
            Some(Box::new(Negate(Some(Box::new(Multiply(vec![
                Value(2.0),
                Value(3.0),
            ])))))),
            Some(Box::new(Negate(Some(Box::new(Value(2.0)))))),
        );
        assert_eq!(a.simplify(), Some(Value(3.0)));

        let a: Operation = Divide(
            Some(Box::new(Negate(Some(Box::new(Multiply(vec![
                Text("x".to_string()),
                Value(2.0),
            ])))))),
            Some(Box::new(Negate(Some(Box::new(Value(2.0)))))),
        );
        assert_eq!(
            a.simplify(),
            Some(Divide(
                Some(Box::new(Negate(Some(Box::new(Multiply(vec![
                    Text("x".to_string()),
                    Value(2.0),
                ])))))),
                Some(Box::new(Value(-2.0)))
            ),)
        );
    }

    #[test]
    fn test_summation_simplification() {
        let a: Operation = Sum(vec![Value(2.0), Value(3.0)]);
        assert_eq!(a.simplify(), Some(Value(5.0)));

        let a: Operation = Sum(vec![Value(2.0), Value(3.0), Value(4.0)]);
        assert_eq!(a.simplify(), Some(Value(9.0)));

        let a: Operation = Sum(vec![Value(2.0), Value(3.0), Text("x".to_string())]);
        assert_eq!(
            a.simplify(),
            Some(Sum(vec![Value(5.0), Text("x".to_string())]))
        );

        let a: Operation = Sum(vec![
            Value(2.0),
            Value(3.0),
            Text("x".to_string()),
            Text("y".to_string()),
        ]);
        assert_eq!(
            a.simplify(),
            Some(Sum(vec![
                Value(5.0),
                Text("x".to_string()),
                Text("y".to_string())
            ]))
        );

        let a: Operation = Sum(vec![
            Value(2.0),
            Value(3.0),
            Text("x".to_string()),
            Text("y".to_string()),
            Value(4.0),
        ]);
        assert_eq!(
            a.simplify(),
            Some(Sum(vec![
                Value(9.0),
                Text("x".to_string()),
                Text("y".to_string())
            ]))
        );

        let a: Operation = Sum(vec![
            Value(0.0),
            Text("x".to_string()),
            Text("y".to_string()),
        ]);
        assert_eq!(
            a.simplify(),
            Some(Sum(vec![Text("x".to_string()), Text("y".to_string())]))
        );

        let a: Operation = Sum(vec![Value(0.0), Text("x".to_string())]);
        assert_eq!(a.simplify(), Some(Text("x".to_string())));
    }

    #[test]
    fn test_get_coefficient() {
        let a: Operation = Divide(
            Some(Box::new(Text("x".to_string()))),
            Some(Box::new(Value(3.0))),
        );
        assert_eq!(a.get_coefficient(), Some(1.0 / 3.0));

        let a: Operation = Divide(
            Some(Box::new(Value(2.0))),
            Some(Box::new(Text("x".to_string()))),
        );
        assert_eq!(a.get_coefficient(), Some(2.0));

        let a: Operation = Divide(
            Some(Box::new(Negate(Some(Box::new(Multiply(vec![
                Value(2.0),
                Value(3.0),
            ])))))),
            Some(Box::new(Text("x".to_string()))),
        );
        assert_eq!(a.get_coefficient(), Some(-6.0));

        let a: Operation = Multiply(vec![Value(2.0), Value(3.0), Text("x".to_string())]);
        assert_eq!(a.get_coefficient(), Some(6.0));
    }
}
