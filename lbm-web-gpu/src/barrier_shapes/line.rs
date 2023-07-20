use std::collections::HashSet;
use::line_drawing::Bresenham;

use super::Shape;

pub struct Line{
    pub points: HashSet<(isize, isize, bool)>
}

impl Shape for Line{
    fn get_points(&self) -> &HashSet<(isize, isize, bool)> {
        &self.points
    }

    fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
} 

impl Line{
    pub fn new(
        end_point_1: (isize, isize), 
        end_point_2: (isize, isize), 
        xdim: isize, 
        ydim: isize) -> Result<Line, String>{
            
            if !Self::validate(end_point_1, end_point_2, xdim, ydim){
                return Err(format!("Endpoints ({},{}) ({},{}) are invalid with dimensions {} and {}", end_point_1.0, end_point_1.1,  end_point_2.0, end_point_2.1,  xdim, ydim));
            }
            
            let mut previous = end_point_1.clone();
            let mut points = HashSet::<(isize, isize, bool)>::new();

            for point in Self::generate_endpoints(end_point_1, end_point_2).iter(){
                for i in Bresenham::new(point.0,  point.1){
                    points.insert((i.0, i.1, true));
                    if Self::diagonal_step(previous,i){
                        let difference = (i.0 - previous.0, i.1 - previous.1);
                        points.insert((i.0 - difference.0, i.1, true));
                        points.insert((i.0, i.1 - difference.1, true));
                    }
                    previous = i;
                }
            }
            
            Ok(
                Line{
                    points
                }
            )
    }

    fn diagonal_step(previous_point: (isize, isize),
                     next_point: (isize, isize)) -> bool{
        previous_point.0 - next_point.0 != 0 && previous_point.1 - next_point.1 != 0
    }

    fn generate_endpoints( end_point_1: (isize, isize), end_point_2: (isize, isize)) -> Vec<((isize, isize), (isize, isize))>{
        
        let mut output = Vec::<((isize, isize), (isize, isize))>::new();
        output.push(Self::order(end_point_1, end_point_2));
        
        if output[0].1.1 == output[0].0.1{
            return output;
        }
        let mut left_endpoints = output[0].clone();
        let mut right_endpoints = output[0].clone();

        if output[0].0.1 > output[0].1.1 { 
            left_endpoints = ((output[0].0.0, output[0].0.1 - 1),(output[0].0.0 + 1, output[0].0.1));
            right_endpoints = ((output[0].1.0 - 1, output[0].1.1),(output[0].1.0, output[0].1.1 - 1)); 
        } else {
            left_endpoints = ((output[0].0.0, output[0].0.1 + 1),(output[0].0.0 + 1, output[0].0.1));
            right_endpoints = ((output[0].1.0 - 1, output[0].1.1),(output[0].1.0, output[0].1.1 - 1));
        }
        output.push((left_endpoints.0, right_endpoints.0));
        output.push((left_endpoints.1, right_endpoints.1));

        output
    }

    fn order(point1: (isize, isize), point2: (isize, isize)) -> ((isize, isize),(isize, isize)){
        if point1.0 > point2.0{
            return (point1, point2);
        }
        (point2, point1)
    }


    fn validate(ep1: (isize, isize), ep2: (isize, isize), xdim: isize, ydim: isize) -> bool{
        ep1.0 >= 0 && ep2.0 >= 0 && ep1.1 >= 0 && ep2.1 >=0 && ep1.0 < xdim && ep2.0 < xdim && ep1.1 < ydim && ep2.1 < ydim
    }

}

