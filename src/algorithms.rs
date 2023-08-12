use crate::equations::Equation;
use crate::operations::Operation;
use crate::operations::Operation::{Add, Divide, Multiply, Subtract, Variable};
use crate::prelude::Value;

pub fn shunting_yard_algorithm(input: String) -> Vec<String> {
    // https://aquarchitect.github.io/swift-algorithm-club/Shunting%20Yard/
    let mut output: Vec<String> = Vec::new();
    let mut operator_stack: Vec<char> = Vec::new();
    let precedence = |c: char| match c {
        '+' | '-' => 1,
        '*' | '/' => 2,
        '^' => 3,
        _ => 0,
    };
    let mut variable_buffer: String = String::new();

    'operator_loop: for x in input.chars() {
        if !variable_buffer.is_empty() {
            if x == '}' {
                variable_buffer += &x.to_string();

                let cleaned: String = variable_buffer
                    .chars()
                    .filter(|x| *x != '{' && *x != '}')
                    .collect();
                if cleaned.parse::<f64>().is_ok() {
                    output.push(cleaned);
                } else {
                    output.push(variable_buffer.clone());
                }

                variable_buffer.clear();
                continue;
            }
            variable_buffer += &x.to_string();
            continue;
        }
        match x {
            // Token is an operator
            '+' | '-' | '*' | '/' => {
                while let Some(y) = operator_stack.pop() {
                    if precedence(y) < precedence(x) {
                        operator_stack.push(y);
                        break;
                    }
                    output.push(String::from(y)); // 2. Add y to output
                }
                operator_stack.push(x); // 2. Push operator onto stack
            }
            // Push Left token onto stack
            '(' => {
                operator_stack.push(x);
            }
            // Pop operators from stack to the output until left token is found
            ')' => {
                while let Some(op) = operator_stack.pop() {
                    if op == '(' {
                        continue 'operator_loop; // Discard left token
                    }
                    output.push(String::from(op));
                }
                panic!("Mismatched parentheses")
            }
            '{' => {
                variable_buffer += &x.to_string();
            }
            // Discard whitespace
            ' ' => {
                continue;
            }
            // Constants and variables
            _ => {
                output.push(String::from(x));
            }
        }
    }
    operator_stack
        .iter()
        .rev()
        .for_each(|x| output.push(String::from(*x)));
    output
}

pub fn binary_tree_algorithm(input: Vec<String>) -> Equation {
    // Section 3: https://www.baeldung.com/cs/postfix-expressions-and-expression-trees
    let mut stack: Vec<Box<Equation>> = Vec::new();

    for x in input {
        match x.as_str() {
            "+" | "-" | "*" | "/" => {
                let right: Option<Box<Equation>> = stack.pop();
                let left: Option<Box<Equation>> = stack.pop();
                let value: Operation;
                match x.as_str() {
                    "+" => value = Add,
                    "-" => value = Subtract,
                    "*" => value = Multiply,
                    "/" => value = Divide,
                    _ => panic!("Unknown operator"),
                }
                let mut equation: Equation = Equation::new(value);
                equation.left = left;
                equation.right = right;
                stack.push(Box::new(equation));
            }
            _ => match x.parse::<f64>().is_ok() {
                true => stack.push(Box::new(Equation::new(Value(x.parse::<f64>().unwrap())))),
                false => stack.push(Box::new(Equation::new(Variable(x.to_string())))),
            },
        }
    }
    *(stack.pop().unwrap())
}

#[cfg(test)]
mod tests {
    use crate::algorithms::{binary_tree_algorithm, shunting_yard_algorithm, Equation};
    use crate::operations::Operation::{Add, Divide, Variable};

    #[test]
    fn test_conversions() {
        let out = shunting_yard_algorithm("(a+b)/c".to_string());
        assert_eq!(out, vec!["a", "b", "+", "c", "/"]);
        let tree: Equation = binary_tree_algorithm(out);
        let expected: Equation = Equation {
            root: Divide,
            left: Some(Box::new(Equation {
                root: Add,
                left: Some(Box::new(Equation::new(Variable("a".to_string())))),
                right: Some(Box::new(Equation::new(Variable("b".to_string())))),
            })),
            right: Some(Box::new(Equation::new(Variable("c".to_string())))),
        };
        assert_eq!(tree, expected);

        let out = shunting_yard_algorithm("A+B*C-D".to_string());
        assert_eq!(out, vec!["A", "B", "C", "*", "+", "D", "-"]);

        let out = shunting_yard_algorithm("4+4*2/(1-5)".to_string());
        assert_eq!(out, vec!["4", "4", "2", "*", "1", "5", "-", "/", "+"]);

        let out = shunting_yard_algorithm("(3+4)*5".to_string());
        assert_eq!(out, vec!["3", "4", "+", "5", "*"]);

        let out = shunting_yard_algorithm("{-1}/c".to_string());
        assert_eq!(out, vec!["-1", "c", "/"]);
    }
}
