use std::collections::HashSet;

use super::Shape;

pub fn merge(
         shapes: Vec<Box<dyn Shape>>, 
         xdim: usize
        ) -> Vec<usize>{
    
    let mut points = HashSet::<(isize, isize)>::new();
    
    for shape in shapes{
        points.extend(shape.get_points());
    }

    points.iter().map(|p| get_index(p, xdim)).collect()
}

fn get_index(point: &(isize, isize), xdim: usize) -> usize{
    point.0 as usize + point.1 as usize * xdim
}