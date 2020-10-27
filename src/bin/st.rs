// Staccato - Statistics from the command line
//
// Copyright 2016-2017 Nick Pillitteri
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

use clap::Clap;
use staccato::{get_values, KeyValueSep, SortingPolicy, StatisticsBundle, StatisticsFormatter};
use std::fs::File;
use std::io::{stdin, BufReader};
use std::process;
use std::str::FromStr;
use std::path::PathBuf;

/// Staccato is a program for generating statistics from a stream
/// of numbers from the command line. It reads values from a file or
/// standard input until the end of the stream (or file) and computes
/// things about them such as the median, mean, standard deviation,
/// and much more.
///
/// By default it will compute statistics for the entire stream. You
/// can have it additionally compute statistics for some subset of the
/// values of the stream. For example, using the argument `-p 25,50`
/// would compute statistics for the lower 25% of values and lower 50%
/// of values.
#[derive(Clap, Debug)]
#[clap(name = "st")]
struct StaccatoOptions {
    /// comma separated list of percentiles (from 1 to 99,
    /// inclusive) that should have metrics computed. Default
    /// is not to compute metrics for any specific percentiles,
    /// only the global metrics.
    #[clap(short = 'p', long)]
    percentiles: Option<Percentiles>,

    /// type of separator to use when printing keys and values.
    /// Possible values for this option are the literal string
    /// 'tab' for the tab character, the literal string 'colon'
    /// for a colon and space, or any other string to use that
    /// as a separator. For example you could use the string
    /// ' => ' as a separator. Default is to use a colon and a
    /// space
    #[clap(short = 's', long)]
    separator: Option<KeyValueSep>,

    /// optional file to read values to from. If not supplied
    /// values will be read from standard input. The values are
    /// expected to be floating point or integer values, one per
    /// line. Leading or trailing whitespace will be removed before
    /// parsing each value.
    #[clap(name = "FILE", parse(from_os_str))]
    file: Option<PathBuf>,
}

#[derive(Default, PartialEq, Debug)]
struct Percentiles {
    value: Vec<u8>,
}

impl FromStr for Percentiles {
    type Err = String;

    fn from_str(val: &str) -> Result<Self, Self::Err> {
        let mut out = Vec::new();
        for p in val.split(",") {
            match p.parse::<u8>() {
                Ok(i) if i > 0 && i < 100 => {
                    out.push(i);
                }
                _ => {
                    return Err(format!("Invalid percentile value {}", p));
                }
            };
        }

        return Ok(Percentiles { value: out });
    }
}

fn main() {
    let opts: StaccatoOptions = StaccatoOptions::parse();
    let percents = opts.percentiles.unwrap_or(Percentiles::default());
    let separator = opts.separator.unwrap_or(KeyValueSep::default());
    let sorting = if !percents.value.is_empty() {
        SortingPolicy::Sorted
    } else {
        SortingPolicy::Unsorted
    };

    let line_result = if let Some(f) = opts.file {
        // If we've been given a file argument, try to open it and read
        // values out of it. If we can't for any reason, just give up and
        // exit now.
        match File::open(f) {
            Ok(handle) => get_values(&mut BufReader::new(handle), sorting),
            Err(e) => {
                eprintln!("error: Cannot open file: {}", e);
                process::exit(1);
            }
        }
    } else {
        // Let the user know we're just going to block on stdin before doing
        // it since sometimes people run commands without arguments just
        // expecting them to display help.
        eprintln!(concat!(
            "notice: waiting for input from stdin. If this isn't what you ",
            "want, try running with the `--help` option"
        ));

        get_values(&mut BufReader::new(stdin()), sorting)
    };

    let lines = match line_result {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: Could not parse values: {}", e);
            process::exit(1);
        }
    };

    let stats = StatisticsBundle::with_percentiles(&lines, &percents.value);
    if let Some(v) = stats {
        print!("{}", StatisticsFormatter::with_sep(&v, separator));
    } else {
        eprintln!("warning: No values to compute stats for");
    }
}

#[cfg(test)]
mod tests {
    use super::Percentiles;
    use std::str::FromStr;

    #[test]
    fn test_parse_percentiles_err_not_in_range() {
        let percents = "75,90,100,110";
        let res = Percentiles::from_str(percents);

        assert!(res.is_err());
    }

    #[test]
    fn test_parse_percentiles_lower_bound() {
        let percents = "0,50,75";
        let res = Percentiles::from_str(percents);

        assert!(res.is_err());
    }

    #[test]
    fn test_parse_percentiles_upper_bound() {
        let percents = "50,75,100";
        let res = Percentiles::from_str(percents);

        assert!(res.is_err());
    }

    #[test]
    fn test_parse_percentiles_err_not_a_number() {
        let percents = "75,banana";
        let res = Percentiles::from_str(percents);

        assert!(res.is_err());
    }

    #[test]
    fn test_parse_percentiles_ok() {
        let percents = "75,90,95,98";
        let res = Percentiles::from_str(percents);

        assert!(res.is_ok());
    }
}
