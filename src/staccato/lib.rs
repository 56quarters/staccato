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
    pub fn from_reader<T: BufRead>(reader: T, percentiles: &[u8])
                                   -> Option<StatisticsBundle>
    {
        let vals = get_sorted_values(reader);
        Self::from_sorted(&vals, percentiles)
    }

    pub fn from_sorted(vals: &[f64], percentiles: &[u8])
                       -> Option<StatisticsBundle>
    {
        if vals.len() == 0 {
            return None;
        }

        let global_stats = Statistics::from(vals, None);
        let percentile_stats = percentiles.iter()
            .map(|&p| Statistics::from(vals, Some(p)))
            .collect();

        Some(StatisticsBundle {
            global: global_stats,
            percentiles: percentile_stats,
        })
    }

    pub fn global_stats(&self) -> &Statistics {
        &self.global
    }

    pub fn percentile_stats(&self) -> &[Statistics] {
        &self.percentiles
    }
}


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
    pub fn from(vals: &[f64], percentile: Option<u8>) -> Statistics {
        let filtered = if let Some(v) = percentile {
            Self::slice_values(vals, v)
        } else {
            vals
        };

        // Bail early when there are no values so that we don't have
        // to handle the 0 case in all the methods to compute stats
        // below.
        if filtered.len() == 0 {
            return Statistics::empty_from_percentile(percentile);
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

    pub fn percentile(&self) -> Option<u8> {
        self.percentile
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn sum(&self) -> f64 {
        self.sum
    }

    pub fn mean(&self) -> f64 {
        self.mean
    }

    pub fn upper(&self) -> f64 {
        self.upper
    }

    pub fn lower(&self) -> f64 {
        self.lower
    }

    pub fn median(&self) -> f64 {
        self.median
    }

    pub fn stddev(&self) -> f64 {
        self.stddev
    }

    fn empty_from_percentile(p: Option<u8>) -> Statistics {
        Statistics {
            percentile: p,
            count: 0,
            sum: 0f64,
            mean: 0f64,
            upper: 0f64,
            lower: 0f64,
            median: 0f64,
            stddev: 0f64,
        }
    }

    fn slice_values(vals: &[f64], percentile: u8) -> &[f64] {
        let num_vals = vals.len();
        let index = (percentile as usize * num_vals) / 100;
        &vals[0..index]
    }

    fn compute_median(vals: &[f64]) -> f64 {
        let len = vals.len();
        let is_odd = len % 2 == 1;

        let median = if is_odd {
            let mid = len / 2;
            vals[mid]
        } else {
            let upper_med = len / 2;
            let lower_med = upper_med - 1;
            // It should never be the case that these indexes
            // don't exist in the slice. If there's no entries
            // we should have already just returned the default
            // stats instance. If there's only one entry we
            // would have handled that with the 'is_odd' case
            // above. Otherwise this will do the right thing.
            (vals[upper_med] + vals[lower_med]) / 2f64
        };

        median
    }

    fn compute_min_max_sum(vals: &[f64]) -> (f64, f64, f64) {
        let mut upper = std::f64::MIN;
        let mut lower = std::f64::MAX;
        let mut sum = 0f64;

        // Compute min, max, and sum in the same method to avoid
        // extra loops through all the values. Thus we only do two
        // loops, this one and the standard deviation loop.
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
        Self::empty_from_percentile(None)
    }
}


impl fmt::Display for Statistics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Should these are write to a String buffer and
        // then write to `f` in a single write?
        if let Some(p) = self.percentile {
            try!(writeln!(f, "count_{}: {}", p, self.count()));
            try!(writeln!(f, "sum_{}: {}", p, self.sum()));
            try!(writeln!(f, "mean_{}: {}", p, self.mean()));
            try!(writeln!(f, "upper_{}: {}", p, self.upper()));
            try!(writeln!(f, "lower_{}: {}", p, self.lower()));
            try!(writeln!(f, "median_{}: {}", p, self.median()));
            try!(writeln!(f, "stddev_{}: {}", p, self.stddev()));
        } else {
            try!(writeln!(f, "count: {}", self.count()));
            try!(writeln!(f, "sum: {}", self.sum()));
            try!(writeln!(f, "mean: {}", self.mean()));
            try!(writeln!(f, "upper: {}", self.upper()));
            try!(writeln!(f, "lower: {}", self.lower()));
            try!(writeln!(f, "median: {}", self.median()));
            try!(writeln!(f, "stddev: {}", self.stddev()));
        }

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::Statistics;

    const VALUES: &'static [f64] = &[
        1f64, 2f64, 5f64, 7f64, 9f64, 12f64
    ];

    const SINGLE: &'static [f64] = &[13f64];

    const EMPTY: &'static [f64] = &[];

    #[test]
    fn test_statistics_full_values_count() {
        let stats = Statistics::from(VALUES, None);
        assert_eq!(6, stats.count());
    }

    #[test]
    fn test_statistics_full_values_sum() {
        let stats = Statistics::from(VALUES, None);
        assert_eq!(36f64, stats.sum());
    }

    #[test]
    fn test_statistics_full_values_mean() {
        let stats = Statistics::from(VALUES, None);
        assert_eq!(6f64, stats.mean());
    }

    #[test]
    fn test_statistics_full_values_upper() {
        let stats = Statistics::from(VALUES, None);
        assert_eq!(12f64, stats.upper());
    }

    #[test]
    fn test_statistics_full_values_lower() {
        let stats = Statistics::from(VALUES, None);
        assert_eq!(1f64, stats.lower());
    }

    #[test]
    fn test_statistics_full_values_median() {
        let stats = Statistics::from(VALUES, None);
        assert_eq!(6f64, stats.median());
    }

    #[test]
    fn test_statistics_full_values_stddev() {
        let stats = Statistics::from(VALUES, None);
        assert!((3.83 - stats.stddev()).abs() < 0.01);
    }

    #[test]
    fn test_statistics_50_values_count() {
        let stats = Statistics::from(VALUES, Some(50));
        assert_eq!(3, stats.count());
    }

    #[test]
    fn test_statistics_50_values_sum() {
        let stats = Statistics::from(VALUES, Some(50));
        assert_eq!(8f64, stats.sum());
    }

    #[test]
    fn test_statistics_50_values_mean() {
        let stats = Statistics::from(VALUES, Some(50));
        assert!((2.66 - stats.mean()).abs() < 0.01);
    }

    #[test]
    fn test_statistics_50_values_upper() {
        let stats = Statistics::from(VALUES, Some(50));
        assert_eq!(5f64, stats.upper());
    }

    #[test]
    fn test_statistics_50_values_lower() {
        let stats = Statistics::from(VALUES, Some(50));
        assert_eq!(1f64, stats.lower());
    }

    #[test]
    fn test_statistics_50_values_median() {
        let stats = Statistics::from(VALUES, Some(50));
        assert_eq!(2f64, stats.median());
    }

    #[test]
    fn test_statistics_50_values_stddev() {
        let stats = Statistics::from(VALUES, Some(50));
        assert!((1.70 - stats.stddev()).abs() < 0.01);
    }

    #[test]
    fn test_statistics_empty_values_count() {
        let stats = Statistics::from(EMPTY, None);
        assert_eq!(0, stats.count());
    }

    #[test]
    fn test_statistics_empty_values_sum() {
        let stats = Statistics::from(EMPTY, None);
        assert_eq!(0f64, stats.sum());
    }

    #[test]
    fn test_statistics_empty_values_mean() {
        let stats = Statistics::from(EMPTY, None);
        assert_eq!(0f64, stats.mean());
    }

    #[test]
    fn test_statistics_empty_values_upper() {
        let stats = Statistics::from(EMPTY, None);
        assert_eq!(0f64, stats.upper());
    }

    #[test]
    fn test_statistics_empty_values_lower() {
        let stats = Statistics::from(EMPTY, None);
        assert_eq!(0f64, stats.lower());
    }

    #[test]
    fn test_statistics_empty_values_median() {
        let stats = Statistics::from(EMPTY, None);
        assert_eq!(0f64, stats.median());
    }

    #[test]
    fn test_statistics_empty_values_stddev() {
        let stats = Statistics::from(EMPTY, None);
        assert_eq!(0f64, stats.stddev());
    }

    #[test]
    fn test_statistics_single_value_count() {
        let stats = Statistics::from(SINGLE, None);
        assert_eq!(1, stats.count());
    }

    #[test]
    fn test_statistics_single_value_sum() {
        let stats = Statistics::from(SINGLE, None);
        assert_eq!(13f64, stats.sum());
    }

    #[test]
    fn test_statistics_single_value_mean() {
        let stats = Statistics::from(SINGLE, None);
        assert_eq!(13f64, stats.mean());
    }

    #[test]
    fn test_statistics_single_value_upper() {
        let stats = Statistics::from(SINGLE, None);
        assert_eq!(13f64, stats.upper());
    }

    #[test]
    fn test_statistics_single_value_lower() {
        let stats = Statistics::from(SINGLE, None);
        assert_eq!(13f64, stats.lower());
    }

    #[test]
    fn test_statistics_single_value_median() {
        let stats = Statistics::from(SINGLE, None);
        assert_eq!(13f64, stats.median());
    }

    #[test]
    fn test_statistics_single_value_stddev() {
        let stats = Statistics::from(SINGLE, None);
        assert_eq!(0f64, stats.stddev());
    }
}
