use std::collections::HashSet;

use::line_drawing::Bresenham;

use super::Shape;

pub struct Line{
    pub points: HashSet<(isize, isize)>
}

impl Shape for Line{
    fn get_points(&self) -> &HashSet<(isize, isize)> {
        &self.points
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
            let mut points = HashSet::<(isize, isize)>::new();
            for i in Bresenham::new(end_point_1, end_point_2){
                points.insert(i);
                if Self::diagonal_step(previous,i){
                    let difference = (i.0 - previous.0, i.1 - previous.1);
                    points.insert((previous.0 + difference.0, i.1));
                    points.insert((i.0, previous.1 + difference.1));
                }
                previous = i;
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


    fn validate(ep1: (isize, isize), ep2: (isize, isize), xdim: isize, ydim: isize) -> bool{
        ep1.0 >= 0 && ep2.0 >= 0 && ep1.1 >= 0 && ep2.1 >=0 && ep1.0 < xdim && ep2.0 < xdim && ep1.1 < ydim && ep2.1 < ydim
    }

}

