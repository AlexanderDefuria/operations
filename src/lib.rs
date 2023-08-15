#![allow(illegal_floating_point_literal_pattern)]

pub mod mappings;
pub mod math;
pub mod operations;

pub mod prelude {
    pub use crate::math::*;
    pub use crate::operations::Operation::*;
    pub use crate::operations::*;
}
