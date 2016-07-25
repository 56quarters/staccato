// stats-rs - Statistics from the command line
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

extern crate getopts;

use std::env;
use std::cmp::Ordering;
use std::fmt;
use std::io::{stdin, stderr, BufRead, BufReader, Write};
use std::process;
use getopts::Options;


const DEFAULT_PERCENTILES: &'static [u8] = &[75, 90, 95, 99];

#[cfg(windows)]
const NL: &'static str = "\r\n";

#[cfg(not(windows))]
const NL: &'static str = "\n";


///
pub enum ErrorPolicy {
    Ignore,
    Mean,
    Median,
    Value,
}


///
///
#[derive(Debug)]
pub struct Statistics {
    percentile: Option<u8>,
    count: usize,
    mean: f64,
    upper: f64,
    lower: f64,
    median: f64,
}


impl Statistics {
    ///
    ///
    ///
    pub fn from(vals: &[f64], percentile: Option<u8>) -> Statistics {
        let mut our_vals = Vec::from(if let Some(v) = percentile {
            Self::slice_values(vals, v)
        } else {
            vals
        });

        our_vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Less));

        let count = our_vals.len();
        let mean = Self::compute_mean(&our_vals);
        let median = Self::compute_median(&our_vals);
        let upper = Self::compute_upper(&our_vals);
        let lower = Self::compute_lower(&our_vals);

        Statistics {
            percentile: percentile,
            count: count,
            mean: mean,
            upper: upper,
            lower: lower,
            median: median,
        }
    }

    ///
    pub fn count(&self) -> usize {
        self.count
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

    ///
    fn slice_values(vals: &[f64], percentile: u8) -> &[f64] {
        let num_vals = vals.len();

        // Pick the end index for each percentile that we've been asked to
        // compute. Use some basic math to avoid having to deal with any
        // floating point operations or numbers at all. For example, if
        // p = 90, n = the number of entries in the vector of values, and
        // x is the desired index for the 90th percentile:
        //
        //  90 * n     90      n
        // -------- = ----- * --- = 0.9 * n = x
        //    100      100     1
        //
        let index = (percentile as usize * num_vals) / 100;
        &vals[0..index]
    }

    ///
    fn compute_mean(vals: &[f64]) -> f64 {
        let num = vals.len() as f64;
        if num == 0f64 {
            return 0f64;
        }

        let sum = vals.iter().fold(0f64, |mut sum, &x| {
            sum = sum + x; sum
        });

        sum / num
    }

    ///
    fn compute_median(vals: &[f64]) -> f64 {
        let mid = vals.len() / 2;
        let med = vals.get(mid);
        *med.unwrap_or(&0f64)
    }

    fn compute_upper(vals: &[f64]) -> f64 {
        let mut upper = std::f64::MIN;
        for &val in vals {
            if val > upper {
                upper = val
            }
        }

        upper
    }

    fn compute_lower(vals: &[f64]) -> f64 {
        let mut lower = std::f64::MAX;
        for &val in vals {
            if val < lower {
                lower = val
            }
        }

        lower
    }
}


impl fmt::Display for Statistics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(p) = self.percentile {
            try!(write!(f, "count_{}: {}{}", p, self.count(), NL));
            try!(write!(f, "mean_{}: {}{}", p, self.mean(), NL));
            try!(write!(f, "upper_{}: {}{}", p, self.upper(), NL));
            try!(write!(f, "lower_{}: {}{}", p, self.lower(), NL));
            try!(write!(f, "median_{}: {}{}", p, self.median(), NL));
        } else {
            try!(write!(f, "count: {}{}", self.count(), NL));
            try!(write!(f, "mean: {}{}", self.mean(), NL));
            try!(write!(f, "upper: {}{}", self.upper(), NL));
            try!(write!(f, "lower: {}{}", self.lower(), NL));
            try!(write!(f, "median: {}{}", self.median(), NL));
        }

        Ok(())
    }
}


fn get_values<T: BufRead>(reader: T) -> (Vec<Option<f64>>, Vec<f64>) {
    let vals: Vec<Option<f64>> = reader.lines()
        .flat_map(|v| v.ok())
        .map(|v| v.parse::<f64>().ok())
        .collect();

    let filtered: Vec<f64> = vals.iter()
        .filter_map(|&v| v)
        .collect();

    (vals, filtered)
}


fn get_usage(prog: &str, opts: &Options) -> String {
    let brief = format!("Usage: {} [options]", prog);
    opts.usage(&brief)
}


fn get_percents(pcnt: String) -> Vec<u8> {
    pcnt.split(",")
        .flat_map(|v| v.parse::<u8>().ok())
        .filter(|&v| v > 0 && v < 100)
        .collect()
}


fn main() {
    let args = env::args().collect::<Vec<String>>();
    let prog = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("p", "percentiles", "Comma separated list of the percentiles \
                                     to compute, numbers between 1 and 99",
                "PCNT");
    opts.optopt("e", "errors", "How to handle malformed input. Options \
                                are to 'ignore' it, replace it with the \
                                'median' value of valid input, replace it \
                                with the 'mean' value of valid input, or \
                                replace it with a particular value.", "ERR");
    opts.optflag("h", "help", "Print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(v) => v,
        Err(e) => {
            write!(stderr(), "Could not parse arguments: {}", e).unwrap();
            process::exit(1);
        }
    };

    if matches.opt_present("h") {
        println!("{}", get_usage(&prog, &opts));
        process::exit(0);
    }

    let reader = BufReader::new(stdin());
    let (vals, filtered) = get_values(reader);

    let global_stats = Statistics::from(&filtered, None);
    print!("{}", global_stats);

    let percents: Vec<u8> = if let Some(p) = matches.opt_str("p") {
        get_percents(p)
    } else {
        Vec::from(DEFAULT_PERCENTILES)
    };

    for p in percents {
        print!("{}", Statistics::from(&filtered, Some(p as u8)));
    }
}
