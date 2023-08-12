#![allow(illegal_floating_point_literal_pattern)]

pub mod algorithms;
pub mod equations;
pub mod operations;

pub mod prelude {
    pub use crate::algorithms::*;
    pub use crate::equations::*;
    pub use crate::operations::Operation::*;
    pub use crate::operations::*;
}
