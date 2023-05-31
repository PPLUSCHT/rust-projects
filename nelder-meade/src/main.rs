mod example_functions;
mod point;
mod simplex;
mod helper_functions;
mod nelder_meade;

use nelder_meade::NelderMeade;

use crate::example_functions::{dot_product, exponential};
use crate::point::Point;
use crate::simplex::Simplex;
use crate::helper_functions::{add_to_slice, distance};
fn main() {
   let guess = vec![1.0; 3].into_boxed_slice();
   let simplex = Simplex::from_guess(guess, 10.0, example_functions::dot_product).unwrap();
   let mut nelder_meade = NelderMeade{
    simplex: simplex,
    reflection: 1.0,
    expansion: 2.0,
    contraction: 0.5,
    shrink: 0.5,
    func: dot_product
   };
   nelder_meade.iterate_until_f_tol(0.01);
   println!("{}", nelder_meade.simplex);
}
