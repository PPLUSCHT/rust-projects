use std::fmt;

use crate::{point::Point, helper_functions::{add_to_index, add_to_slice, multiply_by_const}};

pub struct Simplex{
    pub points: Box<[Point]>
}

impl Simplex{
    //needs to add colinearaity cond!
    pub fn new(mut points: Vec<Vec<f64>>, func: fn(&[f64]) -> f64) -> Result<Simplex, String>{

        let num_point = points.len();
        
        let mut points_vector:Vec<Point> = Vec::with_capacity(num_point);
        
        if num_point <= 2{
            return Err("Point vector is too short! Must contain n+1 points of exactly dimension n. n must be at least 2".to_owned());
        }
        
        while !points.is_empty() {
            let point_vector = match points.pop(){
                None => return Err("Pop from point_vector failed".to_owned()),
                Some(p) => p
            };
            if point_vector.len() + 1 != num_point{
                return Err(format!("Length mismatch in positon {}. Either this vector is the wrong 
                    length ({}) or there are too many or too few 
                    simplex vectors {}", points.len(), point_vector.len(), num_point));
            } else {
                let value = func(&point_vector);
                points_vector.push(
                    Point{
                    x: point_vector.into_boxed_slice(), 
                    value: value
                });
            }
        }

        return Ok(Simplex{points: points_vector.into_boxed_slice()});
    }
    
    pub fn from_guess(point: Box<[f64]>, step: f64, func: fn(&[f64]) -> f64) -> Result<Simplex,String>{
        let dimensions: usize = point.len();
        let mut points_vector:Vec<Point> = Vec::with_capacity(dimensions + 1);
        let mut centroid:Box<[f64]> = point;
        let mut temp_point:Box<[f64]> = centroid.clone();
        let mut radius: f64 = step;
        for i in 0..dimensions{

            temp_point = match add_to_index(centroid.clone(), i, radius){
                Err(str) => panic!("{}", str),
                Ok(point) => point
            };

            centroid = match add_to_index(centroid, i, radius * -0.5){
                Err(str) => panic!("{}", str),
                Ok(point) => point
            };

            radius *= 3.0_f64.powf(0.5)/2.0;

            points_vector.push(Point{
                x: temp_point.clone(),
                value: func(&temp_point)
            });

        }
        temp_point[dimensions - 1] *= -1.0;
        
        let last_value = func(&temp_point);

        points_vector.push(Point{ 
            x: temp_point, 
            value: last_value
        });
        return Ok(Simplex { points: points_vector.into_boxed_slice() });
    }

}

impl fmt::Display for Simplex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result{
        let mut representation: String = String::new();
        representation.push_str("Points: \n");
        for i in self.points.iter(){
            representation.push_str(&i.to_string());
            representation.push('\n');
            representation.push('\n');
        }
        write!(f, "{}", representation)
    }
}