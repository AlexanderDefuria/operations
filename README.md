# Math Operations
This is a Rust library for performing mathematical operations and manipulating equations.

### Installation
Add the following to your `Cargo.toml` file:
```toml
[dependencies]
operations = "0.1.1"
```
### Usage
```rust
use math::{Operation, EquationMember};

fn main() {
    let operation = Operation::Multiply(vec![
        Operation::Value(2.0),
        Operation::Variable(Rc::new("x")),
    ]);

    println!("{}", operation.equation_repr());
}

// Prints 
"2.0 * x"
```

### Features
- Supports basic mathematical operations such as addition, subtraction, multiplication, and division.
- Can manipulate equations by rearranging terms and solving for variables.
- Provides a trait for custom equation members, allowing for easy integration with other libraries.
### Contributing
Contributions are welcome! Open a PR or issue on GitHub to get started!

### License
This library is licensed under the MIT License.