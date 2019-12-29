#![feature(test)]
extern crate staccato;
extern crate test;

use std::fs::File;
use std::io::BufReader;
use test::Bencher;

const SMALL_FILE: &'static str = "benches/values-small.log";
const MED_FILE: &'static str = "benches/values-med.log";
const LARGE_FILE: &'static str = "benches/values-large.log";

fn get_test_values(path: &str) -> Vec<f64> {
    let reader = File::open(path).unwrap();
    let mut buf = BufReader::new(reader);
    staccato::get_values(&mut buf, staccato::SortingPolicy::Sorted).unwrap()
}

#[bench]
fn test_statistics_small_from_sliced_values(b: &mut Bencher) {
    let values = get_test_values(SMALL_FILE);
    b.iter(|| staccato::Statistics::from(&values, Some(75)));
}

#[bench]
fn test_statistics_small_from_all_values(b: &mut Bencher) {
    let values = get_test_values(SMALL_FILE);
    b.iter(|| staccato::Statistics::from(&values, None));
}

#[bench]
fn test_statistics_med_from_sliced_values(b: &mut Bencher) {
    let values = get_test_values(MED_FILE);
    b.iter(|| staccato::Statistics::from(&values, Some(75)));
}

#[bench]
fn test_statistics_med_from_all_values(b: &mut Bencher) {
    let values = get_test_values(MED_FILE);
    b.iter(|| staccato::Statistics::from(&values, None));
}

#[bench]
fn test_statistics_large_from_sliced_values(b: &mut Bencher) {
    let values = get_test_values(LARGE_FILE);
    b.iter(|| staccato::Statistics::from(&values, Some(75)));
}

#[bench]
fn test_statistics_large_from_all_values(b: &mut Bencher) {
    let values = get_test_values(LARGE_FILE);
    b.iter(|| staccato::Statistics::from(&values, None));
}
