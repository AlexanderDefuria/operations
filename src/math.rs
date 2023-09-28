use crate::operations::Operation;
use nalgebra::{DMatrix, DVector};
use ndarray::{Array2, ArrayBase, Ix2, OwnedRepr};
use std::fmt::Debug;
use std::rc::Rc;

pub trait EquationMember {
    /// Returns a string representation of the equation
    fn equation_repr(&self) -> String;

    /// Returns the numeric value of the equation
    fn value(&self) -> f64 {
        f64::NAN
    }

    /// Returns a simplified version of the equation reducing the
    /// number of operations involved
    fn simplify(&self) -> Option<Operation> {
        None
    }

    /// Returns true if the value of the equation is zero
    /// This is important for the required array solver traits
    fn is_zero(&self) -> bool {
        self.value().is_zero()
    }

    /// Returns a latex representation of the equation for the front end
    fn latex_string(&self) -> String {
        self.equation_repr()
    }

    /// Returns Some(Operation) if the equation is representable as an operation
    fn as_operation(&self) -> Option<&Operation> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct EquationRepr {
    string: String,
    latex: Option<String>,
    value: f64,
}

pub(crate) trait EquationSolver {
    fn solve(&self) -> Result<f64, String>;
    fn simplify(&self) -> Result<Equation, String>;
}

#[derive(Debug, Clone)]
pub struct Equation {
    left: Operation,
    right: Operation,
}

impl EquationMember for Equation {
    fn equation_repr(&self) -> String {
        format!(
            "{} = {}",
            self.left.equation_repr(),
            self.right.equation_repr()
        )
    }

    fn value(&self) -> f64 {
        self.left.value() - self.right.value()
    }

    fn simplify(&self) -> Option<Operation> {
        None
    }

    fn as_operation(&self) -> Option<&Operation> {
        None
    }
}

impl EquationMember for EquationRepr {
    fn equation_repr(&self) -> String {
        self.string.clone()
    }
    fn value(&self) -> f64 {
        self.value
    }
    fn latex_string(&self) -> String {
        match &self.latex {
            Some(latex) => latex.clone(),
            None => self.equation_repr(),
        }
    }
}

impl EquationRepr {
    pub fn new(string: String, value: f64) -> EquationRepr {
        EquationRepr {
            string,
            latex: None,
            value,
        }
    }

    pub fn new_with_latex(string: String, latex: String, value: f64) -> EquationRepr {
        EquationRepr {
            string,
            latex: Some(latex),
            value,
        }
    }
}

impl EquationMember for f64 {
    fn equation_repr(&self) -> String {
        let rounded = (self * 1000.0).round() / 1000.0;
        rounded.to_string()
    }
    fn value(&self) -> f64 {
        *self
    }
}

impl EquationMember for usize {
    fn equation_repr(&self) -> String {
        format!("Map({})", self)
    }

    fn value(&self) -> f64 {
        *self as f64
    }
}

impl EquationMember for ArrayBase<OwnedRepr<Operation>, Ix2> {
    fn equation_repr(&self) -> String {
        matrix_to_latex(self.clone())
    }
}

impl<T: EquationMember> EquationMember for DVector<T> {
    fn equation_repr(&self) -> String {
        let mut output: String = String::new();
        output.push_str("\\begin{bmatrix}");
        self.iter().for_each(|x| {
            output.push_str(&x.latex_string());
            output.push_str("\\\\");
        });
        output.push_str("\\end{bmatrix}");
        output
    }
}

impl<T: EquationMember> EquationMember for DMatrix<T> {
    fn equation_repr(&self) -> String {
        let mut output: String = String::new();
        output.push_str("\\begin{bmatrix}");
        self.row_iter().for_each(|x| {
            x.iter().enumerate().for_each(|(i, y)| {
                output.push_str(&y.equation_repr());
                if i != x.len() - 1 {
                    output.push_str(" & "); // Don't add & to last element
                }
            });
            output.push_str("\\\\");
        });
        output.push_str("\\end{bmatrix}");
        output
    }
}

pub fn matrix_to_latex(matrix: Array2<Operation>) -> String {
    let mut latex_a_matrix = String::new();
    latex_a_matrix.push_str("\\begin{bmatrix}");
    for row in matrix.rows() {
        for (i, math) in row.iter().enumerate() {
            latex_a_matrix.push_str(&math.latex_string());
            if i != row.len() - 1 {
                latex_a_matrix.push_str(" & "); // Don't add & to last element
            }
        }
        latex_a_matrix.push_str("\\\\"); // End of row
    }
    latex_a_matrix.push_str("\\end{bmatrix}");
    latex_a_matrix
}

impl<T> From<Rc<T>> for EquationRepr
where
    T: EquationMember,
{
    fn from(rc: Rc<T>) -> Self {
        EquationRepr::new_with_latex(rc.equation_repr(), rc.latex_string(), rc.value())
    }
}

impl EquationMember for (String, f64) {
    fn equation_repr(&self) -> String {
        self.0.clone()
    }
    fn value(&self) -> f64 {
        self.1
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test() {}
}
