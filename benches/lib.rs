#![feature(test)]
extern crate test;
extern crate staccato;

use test::Bencher;
use staccato::Statistics;


const VALUES: &'static [f64] = &[
    0.094f64,
    0.111f64,
    0.183f64,
    0.197f64,
    0.254f64,
    0.271f64,
    0.302f64,
    0.366f64,
    0.453f64,
    0.494f64,
    0.533f64,
    0.542f64,
    0.565f64,
    0.568f64,
    0.582f64,
    0.595f64,
    0.815f64,
    0.912f64,
    0.962f64,
    0.974f64,
];


#[bench]
fn test_statistics_from_sliced_values(b: &mut Bencher) {
    b.iter(|| Statistics::from(VALUES, Some(75)));
}

#[bench]
fn test_statistics_from_all_values(b: &mut Bencher) {
    b.iter(|| Statistics::from(VALUES, None));
}
