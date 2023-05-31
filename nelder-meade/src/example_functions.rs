
pub fn exponential(coords : &[f64]) -> f64{
    let mut exponent : f64 = 0.0;
    for x in coords{
        exponent += x * x;
    }
    std::f64::consts::E.powf(exponent)
}

pub fn dot_product(coords: &[f64]) -> f64{
    let mut total : f64 = 0.0;
    for x in coords{
        total += x * x;
    }
    total
}
