use std::collections::HashSet;

pub mod line;
pub mod merge_shapes;

pub trait Shape {
    fn get_points(&self) -> &HashSet<(isize, isize)>;
}