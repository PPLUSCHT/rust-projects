use crate::{simplex::Simplex, helper_functions::{add_to_slice, multiply_by_const, sub_to_slice, distance, absolute_value}, point::Point};

pub struct NelderMeade{
    pub simplex: Simplex,
    pub reflection: f64,
    pub expansion: f64,
    pub contraction: f64,
    pub shrink: f64,
    pub func: fn(&[f64]) -> f64,
}

impl NelderMeade{

    fn centroid_without_index(&self, index: usize) -> Box<[f64]>{
        let mut centroid: Box<[f64]> = vec![0.0; self.simplex.points.len() - 1].into_boxed_slice();
        for i in 0..self.simplex.points.len() {
            if i != index{
                centroid = match add_to_slice(centroid, &self.simplex.points[i].x){
                    Ok(s) => s,
                    Err(str) => panic!("{}", str)
                };
            }
        }
        multiply_by_const(centroid, 1.0/((self.simplex.points.len() - 1) as f64))
    }
    
    fn centroid(&self) -> Box<[f64]>{
        let mut centroid = self.simplex.points[0].x.clone();
        for i in 1..self.simplex.points.len() {
            centroid = match add_to_slice(centroid, &self.simplex.points[i].x){
                Ok(s) => s,
                Err(str) => panic!("{}", str)
            };
        }
        multiply_by_const(centroid, 1.0/(self.simplex.points.len() as f64))
    }
    
    pub fn needed_points(&self) -> [usize; 3]{
        // needed_indices: [smallest, second_largest, largest]
        let mut needed_points: [usize; 3] = [0,1,2];
        needed_points.sort_by(|a,b| self.simplex.points[*a].value.partial_cmp(&self.simplex.points[*b].value).unwrap());

        for i in 3..self.simplex.points.len(){
            if self.simplex.points[i].value > self.simplex.points[needed_points[2]].value{
                needed_points[1] = needed_points[2];
                needed_points[2] = i;
            } else if self.simplex.points[i].value > self.simplex.points[needed_points[1]].value{
                needed_points[1] = i;
            } else if self.simplex.points[i].value < self.simplex.points[needed_points[0]].value{
                needed_points[0] = i;
            }
        }
        needed_points
    }

    fn reflect(&self, centroid: &[f64], largest_index: usize) -> Box<[f64]>{
        let mut reflected_point = sub_to_slice(centroid.clone().into(), &self.simplex.points[largest_index].x).unwrap();
        reflected_point = multiply_by_const(reflected_point, self.reflection);
        add_to_slice(reflected_point, centroid).unwrap()
    }

    fn expansion(&self, centroid: &[f64], reflection: &[f64]) -> Box<[f64]>{
        let mut expansion_point: Box<[f64]> = sub_to_slice( reflection.clone().into(), centroid).unwrap();
        expansion_point = multiply_by_const(expansion_point, self.expansion);
        add_to_slice(expansion_point, centroid).unwrap()
    }

    fn contraction(&self, centroid: &[f64], point: &[f64]) -> Box<[f64]>{
        let mut contraction: Box<[f64]> = sub_to_slice(centroid.clone().into(), point).unwrap();
        contraction = multiply_by_const(contraction, self.contraction * -1.0);
        add_to_slice(contraction, centroid).unwrap()
    }

    fn shrink(&self, point: &[f64], smallest: &[f64]) -> Box<[f64]>{
        let mut shrink: Box<[f64]> = sub_to_slice(point.clone().into(), smallest).unwrap();
        shrink = multiply_by_const(shrink, self.shrink);
        add_to_slice(shrink, smallest).unwrap()
    }

    fn shrink_all(&mut self, smallest_index: usize){
        for i in 0..self.simplex.points.len(){
            if i != smallest_index{
                let shrunk_point = self.shrink(&self.simplex.points[i].x, &self.simplex.points[smallest_index].x);
                let shrunk_value = (self.func)(&shrunk_point);
                self.simplex.points[i] = Point{
                    x: shrunk_point,
                    value: shrunk_value 
                }
            }
        }
    }

    pub fn step(&mut self, needed_indices: &[usize; 3]){

        let centroid = self.centroid_without_index(needed_indices[2]);
        let reflected_point = self.reflect(&centroid, needed_indices[2]);
        let reflected_value = (self.func)(&reflected_point);
        if reflected_value < self.simplex.points[needed_indices[1]].value && self.simplex.points[needed_indices[0]].value < reflected_value{
            self.simplex.points[needed_indices[2]] = Point{x: reflected_point, value: reflected_value};

        // expansion
        } else if reflected_value < self.simplex.points[needed_indices[0]].value {
            let expansion_point = self.expansion(&centroid, &reflected_point);
            let expansion_value = (self.func)(&expansion_point);
            if expansion_value > reflected_value{
                self.simplex.points[needed_indices[2]] = Point{x: reflected_point, value: reflected_value};

            }else {
                self.simplex.points[needed_indices[2]] = Point{x: expansion_point, value: expansion_value};
            }
        // outer contraction
        } else if reflected_value < self.simplex.points[needed_indices[2]].value {
            let outer_contraction = self.contraction(&centroid, &reflected_point);
            let outer_contraction_value: f64 = (self.func)(&outer_contraction);
            if outer_contraction_value < reflected_value{
                self.simplex.points[needed_indices[2]] = Point{x: outer_contraction, value: outer_contraction_value};
            }
            else {
                self.shrink_all(needed_indices[0]);
            }
        // inner contraction
        } else if reflected_value > self.simplex.points[needed_indices[2]].value {
            let inner_contraction: Box<[f64]> = self.contraction(&centroid, &self.simplex.points[needed_indices[2]].x);
            let inner_contraction_value: f64 = (self.func)(&inner_contraction);
            if inner_contraction_value < self.simplex.points[needed_indices[2]].value{
                self.simplex.points[needed_indices[2]] = Point{x: inner_contraction, value: inner_contraction_value}; 
            }
            else {
                self.shrink_all(needed_indices[0]);
            }
        }

    }

    pub fn iterate_n_times(&mut self, n: usize){
        for i in 0..n{
            self.step(&self.needed_points())
        }
    }

    pub fn iterate_until_x_tol(&mut self, x_tol: f64){
        let mut distance_change = x_tol + 1.0;
        while distance_change > x_tol{
            let needed_points = self.needed_points();
            let previous_worst = self.simplex.points[needed_points[2]].x.clone();
            self.step(&needed_points);
            distance_change = distance(&previous_worst, &self.simplex.points[needed_points[2]].x).unwrap();
        }
    }

    pub fn iterate_until_f_tol(&mut self, f_tol: f64){
        let mut function_change: f64 = f_tol + 1.0;
        while function_change > f_tol{
            let needed_points = self.needed_points();
            let previous_worst = self.simplex.points[needed_points[2]].value;
            self.step(&needed_points);
            function_change = absolute_value(previous_worst - self.simplex.points[needed_points[2]].value);
            println!("{}", function_change);
        }
    }

}