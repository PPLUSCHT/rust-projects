use std::collections::HashSet;

use super::{Shape, blob::Blob};

pub fn merge(
         shapes: &Vec<&dyn Shape>, 
         xdim: usize
        ) -> Blob{
    
    let mut points = HashSet::<(isize, isize, bool)>::new();
    
    for shape in shapes{
        for p in shape.get_points().iter(){
            points.remove(&(p.0, p.1, !p.2));
            points.insert(p.clone());
        }
    }

    Blob {points}
}

pub fn get_points_vector(shape: &dyn Shape, xdim: usize) -> Vec<u32>{
    shape.get_points().iter().flat_map(|x| vec![get_index(x, xdim), get_value(x)]).collect()
}

fn get_index(point: &(isize, isize, bool), xdim: usize) -> u32{
    point.0 as u32 + point.1 as u32 * xdim as u32
}

fn get_value(point: &(isize, isize, bool)) ->  u32{
    if point.2 == true { 1 } else { 0 }
}