use nalgebra::DMatrix;
use std::collections::HashMap;

pub fn count_elements<T>(matrix: &DMatrix<T>) -> HashMap<T, usize>
where
    T: std::hash::Hash + Eq + Copy
{
    let mut element_count = HashMap::new();

    for i in 0..matrix.nrows() {
        for j in 0..matrix.ncols() {
            let element = matrix[(i, j)];
            *element_count.entry(element).or_insert(0) += 1;
        }
    }

    element_count
}