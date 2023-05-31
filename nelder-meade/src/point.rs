use std::fmt;

#[derive(Clone)]
pub struct Point{
    pub x: Box<[f64]>,
    pub value: f64,
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result{
        let mut representation: String = String::new();
        representation.push_str("Coordinates: ");
        for i in self.x.iter(){
            representation.push_str(&i.to_string());
            representation.push(' ');
        }
        representation.push('\n');
        representation.push_str(&format!("Value: {}", self.value));
        write!(f, "{}", representation)
    }
}