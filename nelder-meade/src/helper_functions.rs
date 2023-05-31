pub fn add_to_slice(mut return_slice: Box<[f64]>, slice_to_add: &[f64]) -> Result<Box<[f64]>, String> {
    if return_slice.len() != slice_to_add.len(){
        return Err(format!("Length mismatch. Length of return slice ({}) does not equal length of slice to add ({})", return_slice.len(), slice_to_add.len()));
    }
    for index in  0..slice_to_add.len(){
        return_slice[index] += slice_to_add[index];
    }
    Ok(return_slice)
}

pub fn sub_to_slice(mut return_slice: Box<[f64]>, slice_to_add: &[f64]) -> Result<Box<[f64]>, String>{
    if return_slice.len() != slice_to_add.len(){
        return Err(format!("Length mismatch. Length of return slice ({}) does not equal length of slice to add ({})", return_slice.len(), slice_to_add.len()));
    }
    for index in  0..slice_to_add.len(){
        return_slice[index] -= slice_to_add[index];
    }
    Ok(return_slice)
}

pub fn multiply_by_const(mut return_slice: Box<[f64]>, constant:f64) -> Box<[f64]>{
    for index in 0..return_slice.len(){
        return_slice[index] *= constant;
    }
    return_slice
}

pub fn add_to_index(mut return_slice: Box<[f64]>, index: usize, step: f64) -> Result<Box<[f64]>, String> {
    if return_slice.len() <= index{
        return Err(format!("Index out of range. Slice (length: {}) does not contain index ({})", return_slice.len(), index));
    }
    return_slice[index] += step;
    return Ok(return_slice); 
}

pub fn distance(p1: &[f64], p2: &[f64]) -> Result<f64, String>{
    if p1.len() != p2.len(){
        return Err(format!("Length mismatch. P1 length:{} P2 length: {}", p1.len(), p2.len()));
    }
    let mut distance = 0.0;
    for i in 0..p1.len(){
        distance += (p1[i] - p2[i]).powi(2);
    }
    Ok(distance.powf(0.5))
}

pub fn absolute_value(a:f64) -> f64{
    if a > 0.0{
        return a
    }
    a * -1.0
}