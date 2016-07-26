// Staccato - Statistics from the command line
//
// Copyright 2016 TSH Labs
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.
//


use std::fmt;
use std::cmp::Ordering;
use std::io::BufRead;


#[cfg(windows)]
pub const NL: &'static str = "\r\n";

#[cfg(not(windows))]
pub const NL: &'static str = "\n";



pub fn get_sorted_values<T: BufRead>(reader: T) -> Vec<f64> {
    let mut vals: Vec<f64> = reader.lines()
        .flat_map(|v| v.ok())
        .filter_map(|v| v.parse::<f64>().ok())
        .collect();

    vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Less));

    vals
}


pub struct StatisticsBundle {
    global: Statistics,
    percentiles: Vec<Statistics>,
}


impl StatisticsBundle {
    pub fn from_reader<T: BufRead>(reader: T, percentiles: &[u8]) -> StatisticsBundle {
        let vals = get_sorted_values(reader);
        Self::from_sorted(&vals, percentiles)
    }

    pub fn from_sorted(vals: &[f64], percentiles: &[u8]) -> StatisticsBundle {
        let global_stats = Statistics::from(vals, None);
        let percentile_stats = percentiles.iter()
            .map(|&p| Statistics::from(vals, Some(p)))
            .collect();

        StatisticsBundle {
            global: global_stats,
            percentiles: percentile_stats,
        }
    }

    pub fn global_stats(&self) -> &Statistics {
        &self.global
    }

    pub fn percentile_stats(&self) -> &[Statistics] {
        &self.percentiles
    }
}


///
///
#[derive(Debug)]
pub struct Statistics {
    percentile: Option<u8>,
    count: usize,
    sum: f64,
    mean: f64,
    upper: f64,
    lower: f64,
    median: f64,
    stddev: f64,
}


impl Statistics {
    ///
    ///
    ///
    pub fn from(vals: &[f64], percentile: Option<u8>) -> Statistics {
        let filtered = if let Some(v) = percentile {
            Self::slice_values(vals, v)
        } else {
            vals
        };

        if filtered.len() == 0 {
            return Statistics::default();
        }

        let count = filtered.len();
        let (lower, upper, sum) = Self::compute_min_max_sum(filtered);
        let mean = sum / count as f64;
        let median = Self::compute_median(filtered);
        let stddev = Self::compute_stddev(filtered, mean);

        Statistics {
            percentile: percentile,
            count: count,
            sum: sum,
            mean: mean,
            upper: upper,
            lower: lower,
            median: median,
            stddev: stddev,
        }
    }

    ///
    pub fn count(&self) -> usize {
        self.count
    }

    ///
    pub fn sum(&self) -> f64 {
        self.sum
    }

    ///
    pub fn mean(&self) -> f64 {
        self.mean
    }

    ///
    pub fn upper(&self) -> f64 {
        self.upper
    }

    ///
    pub fn lower(&self) -> f64 {
        self.lower
    }

    ///
    pub fn median(&self) -> f64 {
        self.median
    }

    pub fn stddev(&self) -> f64 {
        self.stddev
    }

    ///
    fn slice_values(vals: &[f64], percentile: u8) -> &[f64] {
        let num_vals = vals.len();
        let index = (percentile as usize * num_vals) / 100;
        &vals[0..index]
    }

    ///
    fn compute_median(vals: &[f64]) -> f64 {
        let mid = vals.len() / 2;
        let med = vals.get(mid);
        *med.unwrap_or(&0f64)
    }

    fn compute_min_max_sum(vals: &[f64]) -> (f64, f64, f64) {
        let mut upper = std::f64::MIN;
        let mut lower = std::f64::MAX;
        let mut sum = 0f64;

        for &val in vals {
            if val > upper {
                upper = val;
            }

            if val < lower {
                lower = val;
            }

            sum += val;
        }

        (lower, upper, sum)
    }


    fn compute_stddev(vals: &[f64], mean: f64) -> f64 {
        let num = vals.len() as f64;
        let sum_deviance = vals.iter().fold(0f64, |mut sum, &x| {
            sum = sum + (x - mean).powi(2); sum
        });

        let deviance = sum_deviance / num;
        deviance.sqrt()
    }
}


impl Default for Statistics {
    fn default() -> Statistics {
        Statistics {
            percentile: None,
            count: 0,
            sum: 0f64,
            mean: 0f64,
            upper: 0f64,
            lower: 0f64,
            median: 0f64,
            stddev: 0f64,
        }
    }
}


impl fmt::Display for Statistics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(p) = self.percentile {
            try!(write!(f, "count_{}: {}{}", p, self.count(), NL));
            try!(write!(f, "sum_{}: {}{}", p, self.sum(), NL));
            try!(write!(f, "mean_{}: {}{}", p, self.mean(), NL));
            try!(write!(f, "upper_{}: {}{}", p, self.upper(), NL));
            try!(write!(f, "lower_{}: {}{}", p, self.lower(), NL));
            try!(write!(f, "median_{}: {}{}", p, self.median(), NL));
            try!(write!(f, "stddev_{}: {}{}", p, self.stddev(), NL));
        } else {
            try!(write!(f, "count: {}{}", self.count(), NL));
            try!(write!(f, "sum: {}{}", self.sum(), NL));
            try!(write!(f, "mean: {}{}", self.mean(), NL));
            try!(write!(f, "upper: {}{}", self.upper(), NL));
            try!(write!(f, "lower: {}{}", self.lower(), NL));
            try!(write!(f, "median: {}{}", self.median(), NL));
            try!(write!(f, "stddev: {}{}", self.stddev(), NL));
        }

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn test_statistics_full_values_count() {

    }

    #[test]
    fn test_statistics_full_values_mean() {

    }

    #[test]
    fn test_statistics_full_values_upper() {

    }

    #[test]
    fn test_statistics_full_values_lower() {

    }

    #[test]
    fn test_statistics_full_values_median() {

    }

    #[test]
    fn test_statistics_full_values_stddev() {

    }

    #[test]
    fn test_statistics_75_values_count() {

    }

    #[test]
    fn test_statistics_75_values_mean() {

    }

    #[test]
    fn test_statistics_75_values_upper() {

    }

    #[test]
    fn test_statistics_75_values_lower() {

    }

    #[test]
    fn test_statistics_75_values_median() {

    }

    #[test]
    fn test_statistics_75_values_stddev() {

    }
}
