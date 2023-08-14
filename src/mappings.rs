use crate::math::EquationMember;
use crate::prelude::*;
use std::rc::Rc;

/// Create a mapping for operation expansion.
///
/// In a perfect world this would be a constant but that will most likely require an
/// intermediary data structure to be created. This is a temporary solution.
/// We run into this issue with recursive types that need to be represented in the
/// heap and thus cannot be represented as a constant as the heap does not exist at compile time.
fn expansions() -> Vec<(Operation, Operation)> {
    vec![
        (
            Divide(
                Some(Box::new(Sum(vec![Mapping(0), Mapping(1)]))),
                Some(Box::new(Mapping(2))),
            ),
            Sum(vec![
                Divide(Some(Box::new(Mapping(0))), Some(Box::new(Mapping(2)))),
                Divide(Some(Box::new(Mapping(1))), Some(Box::new(Mapping(2)))),
            ]),
        ),
        (
            Divide(
                Some(Box::new(Sum(vec![Mapping(0), Mapping(1), Mapping(2)]))),
                Some(Box::new(Mapping(3))),
            ),
            Sum(vec![
                Divide(Some(Box::new(Mapping(0))), Some(Box::new(Mapping(3)))),
                Divide(Some(Box::new(Mapping(1))), Some(Box::new(Mapping(3)))),
                Divide(Some(Box::new(Mapping(2))), Some(Box::new(Mapping(3)))),
            ]),
        ),
    ]
}

/// Create a mapping index for an operation.
///
/// Recursively traverses the input `Operation`, extracting and collecting all the individual
/// components that are part of the operation's structure. This index is useful for applying
/// a mapping to specific components.
pub(crate) fn create_mapping_index(input: Operation) -> Vec<Operation> {
    let mut output: Vec<Operation> = Vec::new();
    match input {
        Multiply(contents) | Sum(contents) => {
            for x in contents {
                output.extend(create_mapping_index(x));
            }
        }
        Negate(a) => match a {
            Some(a) => output.extend(create_mapping_index(*a)),
            None => {}
        },
        Divide(n, d) => {
            match n {
                Some(n) => output.extend(create_mapping_index(*n)),
                None => {}
            }
            match d {
                Some(d) => output.extend(create_mapping_index(*d)),
                None => {}
            }
        }
        Value(_) | Text(_) | Mapping(_) => output.push(input),
    }

    output
}

/// Apply a mapping to an operation.
///
/// Recursively applies the given `mappings` to the components of the `input` operation,
/// effectively substituting parts of the operation's structure according to the provided mappings.
pub(crate) fn apply_mapping(input: &mut Operation, mappings: Vec<Operation>) -> Operation {
    let mut output: Operation = input.clone();
    match output {
        Multiply(ref mut contents) | Sum(ref mut contents) => {
            contents.iter_mut().for_each(|x| {
                *x = apply_mapping(x, mappings.clone());
            });
        }
        Negate(ref mut a) => match a {
            Some(ref mut a) => **a = apply_mapping(a, mappings),
            None => {}
        },
        Divide(ref mut n, ref mut d) => {
            match n {
                Some(ref mut n) => **n = apply_mapping(n, mappings.clone()),
                None => {}
            }
            match d {
                Some(ref mut d) => **d = apply_mapping(d, mappings),
                None => {}
            }
        }
        Value(_) | Text(_) => output = Value(Rc::new(0.0)),
        Mapping(ref mut index) => {
            return mappings[*index].clone();
        }
    }
    output
}

/// Map a given operation using a set of expansion mappings.
///
/// This function maps the input `Operation` to another operation using the provided `mapping` function.
/// It also checks for predefined expansions and applies them, resulting in a transformed operation.
pub(crate) fn map<'a>(input: Operation, mapping: fn() -> Vec<(Operation, Operation)>, depth: usize) -> Operation {
    let mut output: Operation = input.clone();
    for (a, b) in expansions().iter() {
        if output.compare_structure(a, depth) {
            output = b.clone();
            break;
        }
    }
    let mappings: Vec<Operation> = create_mapping_index(input);
    apply_mapping(&mut output, mappings)
}

/// Expand an operation by applying available mappings.
///
/// This function attempts to expand the given `input` operation by applying predefined expansion
/// mappings. If an expansion is successful, it returns the transformed operation wrapped in `Ok()`.
/// If no expansion is possible, it returns the original operation wrapped in `Err()`.

pub fn expand(input: Operation) -> Result<Operation, Operation> {
    let depth: usize = 2;
    let output: Operation = map(input.clone(), expansions, depth);
    if output.compare_structure(&input, depth) {
        Err(output)
    } else {
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use crate::mappings::{create_mapping_index, expand, map};
    use crate::prelude::*;
    use std::rc::Rc;

    #[test]
    fn test_mapping() {
        let a: Operation = Divide(
            Some(Box::new(Sum(vec![Mapping(0), Mapping(1)]))),
            Some(Box::new(Mapping(4))),
        );
        let b: Operation = Sum(vec![
            Divide(Some(Box::new(Mapping(0))), Some(Box::new(Mapping(4)))),
            Divide(Some(Box::new(Mapping(1))), Some(Box::new(Mapping(4)))),
        ]);
        assert_eq!(expand(a), Ok(b));

        let a: Operation = Divide(
            Some(Box::new(Sum(vec![
                Text("x".to_string()),
                Text("y".to_string()),
            ]))),
            Some(Box::new(Text("z".to_string()))),
        );
        let b: Operation = Sum(vec![
            Divide(
                Some(Box::new(Text("x".to_string()))),
                Some(Box::new(Text("z".to_string()))),
            ),
            Divide(
                Some(Box::new(Text("y".to_string()))),
                Some(Box::new(Text("z".to_string()))),
            ),
        ]);
        assert_eq!(expand(a), Ok(b));

        let a: Operation = Divide(
            Some(Box::new(Sum(vec![
                Text("x".to_string()),
                Text("y".to_string()),
            ]))),
            Some(Box::new(Value(Rc::new(8.0)))),
        );
        let b: Operation = Sum(vec![
            Divide(
                Some(Box::new(Text("x".to_string()))),
                Some(Box::new(Value(Rc::new(8.0)))),
            ),
            Divide(
                Some(Box::new(Text("y".to_string()))),
                Some(Box::new(Value(Rc::new(8.0)))),
            ),
        ]);
        assert_eq!(expand(a), Ok(b));

        let a: Operation = Divide(
            Some(Box::new(Sum(vec![
                Text("x".to_string()),
                Text("y".to_string()),
                Text("z".to_string()),
            ]))),
            Some(Box::new(Value(Rc::new(8.0)))),
        );
        let b: Operation = Sum(vec![
            Divide(
                Some(Box::new(Text("x".to_string()))),
                Some(Box::new(Value(Rc::new(8.0)))),
            ),
            Divide(
                Some(Box::new(Text("y".to_string()))),
                Some(Box::new(Value(Rc::new(8.0)))),
            ),
            Divide(
                Some(Box::new(Text("z".to_string()))),
                Some(Box::new(Value(Rc::new(8.0)))),
            ),
        ]);
        assert_eq!(expand(a), Ok(b));

        let a: Operation = Divide(
            Some(Box::new(Sum(vec![
                Negate(Some(Box::new(Text("x".to_string())))),
                Negate(Some(Box::new(Text("y".to_string())))),
                Negate(Some(Box::new(Text("z".to_string())))),
            ]))),
            Some(Box::new(Value(Rc::new(8.0)))),
        );
        let b: Operation = Sum(vec![
            Divide(
                Some(Box::new(Negate(Some(Box::new(Text("x".to_string())))))),
                Some(Box::new(Value(Rc::new(8.0)))),
            ),
            Divide(
                Some(Box::new(Negate(Some(Box::new(Text("y".to_string())))))),
                Some(Box::new(Value(Rc::new(8.0)))),
            ),
            Divide(
                Some(Box::new(Negate(Some(Box::new(Text("z".to_string())))))),
                Some(Box::new(Value(Rc::new(8.0)))),
            ),
        ]);
        assert_eq!(expand(a), Ok(b));

    }

    #[test]
    fn test_create_mapping_index() {
        let a: Operation = Divide(
            Some(Box::new(Sum(vec![Mapping(0), Mapping(1)]))),
            Some(Box::new(Mapping(2))),
        );
        let b: Vec<Operation> = vec![Mapping(0), Mapping(1), Mapping(2)];
        assert_eq!(create_mapping_index(a), b);

        let a: Operation = Divide(
            Some(Box::new(Sum(vec![
                Text("x".to_string()),
                Text("y".to_string()),
            ]))),
            Some(Box::new(Text("z".to_string()))),
        );
        let b: Vec<Operation> = vec![
            Text("x".to_string()),
            Text("y".to_string()),
            Text("z".to_string()),
        ];
        assert_eq!(create_mapping_index(a), b);

        let a: Operation = Divide(
            Some(Box::new(Sum(vec![
                Text("x".to_string()),
                Text("y".to_string()),
            ]))),
            Some(Box::new(Value(Rc::new(1.0)))),
        );
        let b: Vec<Operation> = vec![
            Text("x".to_string()),
            Text("y".to_string()),
            Value(Rc::new(1.0)),
        ];
        assert_eq!(create_mapping_index(a), b);

        let a: Operation = Multiply(vec![Text("x".to_string()), Text("y".to_string())]);
        let b: Vec<Operation> = vec![Text("x".to_string()), Text("y".to_string())];
        assert_eq!(create_mapping_index(a), b);
    }

    #[test]
    fn test_compare_structure() {
        let a: Operation = Divide(
            Some(Box::new(Sum(vec![Mapping(0), Mapping(1)]))),
            Some(Box::new(Mapping(2))),
        );
        let b: Operation = Divide(
            Some(Box::new(Sum(vec![Mapping(0), Mapping(1)]))),
            Some(Box::new(Mapping(2))),
        );
        assert!(a.compare_entire_structure(&b));

        let b: Operation = Divide(
            Some(Box::new(Sum(vec![
                Text("x".to_string()),
                Text("y".to_string()),
            ]))),
            Some(Box::new(Text("z".to_string()))),
        );
        assert!(a.compare_entire_structure(&b));

        let b: Operation = Divide(
            Some(Box::new(Sum(vec![
                Text("x".to_string()),
                Text("y".to_string()),
            ]))),
            Some(Box::new(Value(Rc::new(1.0)))),
        );
        assert!(a.compare_entire_structure(&b));

        let b: Operation = Divide(
            Some(Box::new(Multiply(vec![
                Text("x".to_string()),
                Text("y".to_string()),
            ]))),
            Some(Box::new(Value(Rc::new(2.0)))),
        );
        assert!(!a.compare_entire_structure(&b));

        let a: Operation = Multiply(vec![Text("x".to_string()), Text("y".to_string())]);
        let b: Operation = Multiply(vec![
            Text("x".to_string()),
            Text("y".to_string()),
            Text("z".to_string()),
        ]);
        assert!(!a.compare_entire_structure(&b));
    }
}
