// Staccato - Statistics from the command line
//
// Copyright 2016-2017 TSH Labs
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
use std::io;
use std::cmp::Ordering;
use std::io::Read;
use std::fmt::Write;
use std::str::FromStr;


const DISPLAY_PRECISION: usize = 5;


#[derive(PartialEq, Eq)]
pub enum SortingPolicy {
    Sorted,
    Unsorted,
}


pub fn get_values<T: Read>(reader: &mut T, sort: SortingPolicy) -> Result<Vec<f64>, io::Error> {
    let mut buf = String::new();
    try!(reader.read_to_string(&mut buf));

    let mut values: Vec<f64> = buf.lines()
        .filter_map(|v| v.parse::<f64>().ok())
        .collect();

    if sort == SortingPolicy::Sorted {
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Less));
    }

    Ok(values)
}


#[derive(Debug, Clone)]
pub struct StatisticsBundle {
    global: Statistics,
    percentiles: Vec<Statistics>,
}


impl StatisticsBundle {

    /// Create a statistics bundle from a sequence of values.
    ///
    /// Note that as opposed to the `with_percentiles` method, there is no
    /// requirement that these values are sorted.
    ///
    /// This method returns `None` if the sequence of values is empty.
    pub fn from(vals: &[f64]) -> Option<StatisticsBundle> {
        Self::with_percentiles(vals, &[])
    }

    /// Create a statistics bundle from a **sorted** sequence of values and
    /// a sequence of percentiles.
    ///
    /// The values must be sorted or the statistics will be incorrect.
    ///
    /// This method returns `None` if the sequence of values is empty.
    /// Additionally, if there are not enough values to create all the
    /// desired percentile slices (e.g. 90th percentile for a series of
    /// only 7 values) the slices without enough values will be omitted.
    pub fn with_percentiles(vals: &[f64], percentiles: &[u8]) -> Option<StatisticsBundle> {
        if vals.len() == 0 {
            return None;
        }

        let percentile_stats = percentiles.iter()
            .flat_map(|&p| Statistics::from(vals, Some(p)))
            .collect();

        Statistics::from(vals, None).map(|global| { StatisticsBundle {
            global: global,
            percentiles: percentile_stats,
        }})
    }

    pub fn global_stats(&self) -> &Statistics {
        &self.global
    }

    pub fn percentile_stats(&self) -> &[Statistics] {
        &self.percentiles
    }
}


#[derive(Debug, Clone)]
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
    pub fn from(vals: &[f64], percentile: Option<u8>) -> Option<Statistics> {
        let filtered = if let Some(v) = percentile {
            Self::slice_values(vals, v)
        } else {
            vals
        };

        // Bail early when there are no values so that we don't have
        // to handle the 0 case in all the methods to compute stats
        // below.
        let count = filtered.len();
        if count == 0 {
            return None
        }

        let (lower, upper, sum) = Self::compute_min_max_sum(filtered);
        let mean = sum / count as f64;
        let median = Self::compute_median(filtered);
        let stddev = Self::compute_stddev(filtered, mean);

        Some(Statistics {
            percentile: percentile,
            count: count,
            sum: sum,
            mean: mean,
            upper: upper,
            lower: lower,
            median: median,
            stddev: stddev,
        })
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

    fn slice_values(vals: &[f64], percentile: u8) -> &[f64] {
        let num_vals = vals.len();
        let index = (percentile as usize * num_vals) / 100;
        &vals[0..index]
    }

    fn compute_median(vals: &[f64]) -> f64 {
        let len = vals.len();
        let is_odd = len % 2 == 1;

        if is_odd {
            let mid = len / 2;
            return vals[mid];
        }

        let upper_med = len / 2;
        let lower_med = upper_med - 1;
        // It should never be the case that these indexes
        // don't exist in the slice. If there's no entries
        // we should have already just returned the default
        // stats instance. If there's only one entry we
        // would have handled that with the 'is_odd' case
        // above. Otherwise this will do the right thing.
        (vals[upper_med] + vals[lower_med]) / 2f64

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
        let sum_deviance = vals.iter().fold(0f64, |sum, &x| {
            sum + (x - mean).powi(2)
        });

        let deviance = sum_deviance / num;
        deviance.sqrt()
    }
}


#[derive(PartialEq, Debug, Clone)]
pub struct KeyValueParseError(());


#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub enum KeyValueSep {
    Tab,
    Colon,
    Other(String),
}


impl KeyValueSep {
    fn get_sep(&self) -> &str {
        match *self {
            KeyValueSep::Tab => "\t",
            KeyValueSep::Colon => ": ",
            KeyValueSep::Other(ref s) => s,
        }
    }
}


impl fmt::Display for KeyValueSep {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.get_sep().fmt(f)
    }
}


impl FromStr for KeyValueSep {
    type Err = KeyValueParseError;

    fn from_str(s: &str) -> Result<KeyValueSep, Self::Err> {
        if "tab" == s {
            Ok(KeyValueSep::Tab)
        } else if "colon" == s {
            Ok(KeyValueSep::Colon)
        } else {
            Ok(KeyValueSep::Other(s.to_string()))
        }
    }
}


#[derive(Debug)]
pub struct StatisticsFormatter<'a> {
    bundle: &'a StatisticsBundle,
    sep: KeyValueSep,
}


impl<'a> StatisticsFormatter<'a> {
    pub fn new(bundle: &'a StatisticsBundle) -> StatisticsFormatter<'a> {
        Self::with_sep(bundle, KeyValueSep::Colon)
    }

    pub fn with_sep(bundle: &'a StatisticsBundle, sep: KeyValueSep) -> StatisticsFormatter<'a> {
        StatisticsFormatter { bundle: bundle, sep: sep }
    }

    fn write_to_buf<T: Write>(buf: &mut T, stats: &Statistics, sep: &KeyValueSep) {
        if let Some(p) = stats.percentile() {
            writeln!(buf, "count_{}{}{:.*}", p, sep, DISPLAY_PRECISION, stats.count()).unwrap();
            writeln!(buf, "sum_{}{}{:.*}", p, sep, DISPLAY_PRECISION, stats.sum()).unwrap();
            writeln!(buf, "mean_{}{}{:.*}", p, sep, DISPLAY_PRECISION, stats.mean()).unwrap();
            writeln!(buf, "upper_{}{}{:.*}", p, sep, DISPLAY_PRECISION, stats.upper()).unwrap();
            writeln!(buf, "lower_{}{}{:.*}", p, sep, DISPLAY_PRECISION, stats.lower()).unwrap();
            writeln!(buf, "median_{}{}{:.*}", p, sep, DISPLAY_PRECISION, stats.median()).unwrap();
            writeln!(buf, "stddev_{}{}{:.*}", p, sep, DISPLAY_PRECISION, stats.stddev()).unwrap();
        } else {
            writeln!(buf, "count{}{:.*}", sep, DISPLAY_PRECISION, stats.count()).unwrap();
            writeln!(buf, "sum{}{:.*}", sep, DISPLAY_PRECISION, stats.sum()).unwrap();
            writeln!(buf, "mean{}{:.*}", sep, DISPLAY_PRECISION, stats.mean()).unwrap();
            writeln!(buf, "upper{}{:.*}", sep, DISPLAY_PRECISION, stats.upper()).unwrap();
            writeln!(buf, "lower{}{:.*}", sep, DISPLAY_PRECISION, stats.lower()).unwrap();
            writeln!(buf, "median{}{:.*}", sep, DISPLAY_PRECISION, stats.median()).unwrap();
            writeln!(buf, "stddev{}{:.*}", sep, DISPLAY_PRECISION, stats.stddev()).unwrap();
        }
    }
}


impl<'a> fmt::Display for StatisticsFormatter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut buf = String::new();

        let global_stats = self.bundle.global_stats();
        Self::write_to_buf(&mut buf, global_stats, &self.sep);

        for stats in self.bundle.percentile_stats() {
            Self::write_to_buf(&mut buf, stats, &self.sep);
        }

        buf.fmt(f)
    }
}


#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use super::{get_values, SortingPolicy, Statistics, KeyValueSep};

    const VALUES: &'static [f64] = &[
        1f64, 2f64, 5f64, 7f64, 9f64, 12f64
    ];

    const SINGLE: &'static [f64] = &[13f64];

    const EMPTY: &'static [f64] = &[];

    #[test]
    fn test_get_values_filter_invalids() {
        let bytes: Vec<u8> = vec!["asdf\n", "4.5\n", "xyz\n"].iter()
            .flat_map(|v| v.as_bytes())
            .map(|&v| v)
            .collect();

        let mut reader = Cursor::new(bytes);
        assert_eq!(vec![4.5], get_values(&mut reader, SortingPolicy::Sorted).unwrap());
    }

    #[test]
    fn test_get_values_ordered() {
        let bytes: Vec<u8> = vec!["9.8\n", "4.5\n", "5.6\n"].iter()
            .flat_map(|v| v.as_bytes())
            .map(|&v| v)
            .collect();

        let mut reader = Cursor::new(bytes);
        assert_eq!(vec![4.5, 5.6, 9.8], get_values(&mut reader, SortingPolicy::Sorted).unwrap());
    }

    #[test]
    fn test_get_values_unordered() {
        let bytes: Vec<u8> = vec!["9.8\n", "4.5\n", "5.6\n"].iter()
            .flat_map(|v| v.as_bytes())
            .map(|&v| v)
            .collect();

        let mut reader = Cursor::new(bytes);
        assert_eq!(vec![9.8, 4.5, 5.6], get_values(&mut reader, SortingPolicy::Unsorted).unwrap());
    }

    #[test]
    fn test_statistics_full_values_count() {
        let stats = Statistics::from(VALUES, None).unwrap();
        assert_eq!(6, stats.count());
    }

    #[test]
    fn test_statistics_full_values_sum() {
        let stats = Statistics::from(VALUES, None).unwrap();
        assert_eq!(36f64, stats.sum());
    }

    #[test]
    fn test_statistics_full_values_mean() {
        let stats = Statistics::from(VALUES, None).unwrap();
        assert_eq!(6f64, stats.mean());
    }

    #[test]
    fn test_statistics_full_values_upper() {
        let stats = Statistics::from(VALUES, None).unwrap();
        assert_eq!(12f64, stats.upper());
    }

    #[test]
    fn test_statistics_full_values_lower() {
        let stats = Statistics::from(VALUES, None).unwrap();
        assert_eq!(1f64, stats.lower());
    }

    #[test]
    fn test_statistics_full_values_median() {
        let stats = Statistics::from(VALUES, None).unwrap();
        assert_eq!(6f64, stats.median());
    }

    #[test]
    fn test_statistics_full_values_stddev() {
        let stats = Statistics::from(VALUES, None).unwrap();
        assert!((3.83 - stats.stddev()).abs() < 0.01);
    }

    #[test]
    fn test_statistics_50_values_count() {
        let stats = Statistics::from(VALUES, Some(50)).unwrap();
        assert_eq!(3, stats.count());
    }

    #[test]
    fn test_statistics_50_values_sum() {
        let stats = Statistics::from(VALUES, Some(50)).unwrap();
        assert_eq!(8f64, stats.sum());
    }

    #[test]
    fn test_statistics_50_values_mean() {
        let stats = Statistics::from(VALUES, Some(50)).unwrap();
        assert!((2.66 - stats.mean()).abs() < 0.01);
    }

    #[test]
    fn test_statistics_50_values_upper() {
        let stats = Statistics::from(VALUES, Some(50)).unwrap();
        assert_eq!(5f64, stats.upper());
    }

    #[test]
    fn test_statistics_50_values_lower() {
        let stats = Statistics::from(VALUES, Some(50)).unwrap();
        assert_eq!(1f64, stats.lower());
    }

    #[test]
    fn test_statistics_50_values_median() {
        let stats = Statistics::from(VALUES, Some(50)).unwrap();
        assert_eq!(2f64, stats.median());
    }

    #[test]
    fn test_statistics_50_values_stddev() {
        let stats = Statistics::from(VALUES, Some(50)).unwrap();
        assert!((1.70 - stats.stddev()).abs() < 0.01);
    }

    #[test]
    fn test_statistics_empty_values() {
        assert!(Statistics::from(EMPTY, None).is_none());
    }

    #[test]
    fn test_statistics_single_value_count() {
        let stats = Statistics::from(SINGLE, None).unwrap();
        assert_eq!(1, stats.count());
    }

    #[test]
    fn test_statistics_single_value_sum() {
        let stats = Statistics::from(SINGLE, None).unwrap();
        assert_eq!(13f64, stats.sum());
    }

    #[test]
    fn test_statistics_single_value_mean() {
        let stats = Statistics::from(SINGLE, None).unwrap();
        assert_eq!(13f64, stats.mean());
    }

    #[test]
    fn test_statistics_single_value_upper() {
        let stats = Statistics::from(SINGLE, None).unwrap();
        assert_eq!(13f64, stats.upper());
    }

    #[test]
    fn test_statistics_single_value_lower() {
        let stats = Statistics::from(SINGLE, None).unwrap();
        assert_eq!(13f64, stats.lower());
    }

    #[test]
    fn test_statistics_single_value_median() {
        let stats = Statistics::from(SINGLE, None).unwrap();
        assert_eq!(13f64, stats.median());
    }

    #[test]
    fn test_statistics_single_value_stddev() {
        let stats = Statistics::from(SINGLE, None).unwrap();
        assert_eq!(0f64, stats.stddev());
    }

    #[test]
    fn test_key_value_sep_get_sep() {
        assert_eq!("\t", KeyValueSep::Tab.get_sep());
        assert_eq!(": ", KeyValueSep::Colon.get_sep());
        assert_eq!(" => ", KeyValueSep::Other(" => ".to_string()).get_sep());
    }

    #[test]
    fn test_key_value_sep_display() {
        assert_eq!("\t".to_string(), format!("{}", KeyValueSep::Tab));
        assert_eq!(": ".to_string(), format!("{}", KeyValueSep::Colon));
        assert_eq!(" => ".to_string(), format!("{}", KeyValueSep::Other(" => ".to_string())));
    }

    #[test]
    fn test_key_value_sep_from_str() {
        assert_eq!(KeyValueSep::Tab, "tab".parse::<KeyValueSep>().unwrap());
        assert_eq!(KeyValueSep::Colon, "colon".parse::<KeyValueSep>().unwrap());
        assert_eq!(KeyValueSep::Other(" => ".to_string()), " => ".parse::<KeyValueSep>().unwrap());

    }
}
