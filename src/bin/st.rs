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

extern crate staccato;
#[macro_use] extern crate clap;

use std::env;
use std::fs::File;
use std::io::{stdin, stderr, BufReader, Write};
use std::process;

use clap::{Arg, App, ArgMatches};
use staccato::{get_sorted_values, StatisticsBundle};


const DEFAULT_PERCENTILES: &'static [u8] = &[];
const DEFAULT_TERM_WIDTH: usize = 72;


fn parse_cli_opts<'a>(args: Vec<String>) -> ArgMatches<'a> {
    App::new("Staccato")
        .version(crate_version!())
        .set_term_width(DEFAULT_TERM_WIDTH)
        .after_help("\
Staccato is a program for generating statistics from a stream \
of numbers from the command line. It reads values from a file or \
standard input until the end of the stream (or file) and computes \
things about them such as the median, mean, standard deviation, \
and much more.

By default it will compute statistics for the entire stream. You can \
have it additionally compute statistics for some subset of the values \
of the stream. For example, using the argument `-p 25,50` would compute \
statistics for the lower 25% of values, and lower 50% of values.

If you've ever used Statsd, the format should seem familiar :)")
        .arg(Arg::with_name("percentiles")
             .short("p")
             .long("percentiles")
             .help(
                 "Comma separated list of percentiles (from 1 to 99, \
                  inclusive) that should have metrics computed. Default \
                  is not to compute metrics for any specific percentiles, \
                  only the global metrics.")
             .takes_value(true)
             .validator(validate_percents))
        // Note that we aren't using any validators for the file input. We'll
        // just try to open it and see what happens. Otherwise we become susceptible
        // to race conditions.
        .arg(Arg::with_name("file")
             .help(
                 "Optional file to read values to from. If not supplied \
                  values will be read from standard input.")
             .index(1))
        .get_matches_from(args)
}


fn validate_percents(v: String) -> Result<(), String> {
    for p in v.split(",") {
        let p_as_u8 = match p.parse::<u8>() {
            Ok(i) => i,
            Err(_) => {
                return Err("Invalid percentile value".to_string());
            }
        };

        if p_as_u8 < 1 || p_as_u8 > 99 {
            return Err("Invalid percentile value".to_string());
        }
    }

    Ok(())
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let matches = parse_cli_opts(args);

    let percents: Vec<u8> = if let Some(p) = matches.values_of("percentiles") {
        p
            .flat_map(|v| v.parse::<u8>().ok())
            .filter(|&v| v >= 1 && v <= 99)
            .collect()
    } else {
        Vec::from(DEFAULT_PERCENTILES)
    };

    let lines = if let Some(f) = matches.value_of("file") {
        // If we've been given a file argument, try to open it and read
        // values out of it. If we can't for any reason, just given up and
        // exit now.
        match File::open(f) {
            Ok(handle) => get_sorted_values(BufReader::new(handle)),
            Err(e) => {
                let _ = writeln!(stderr(), "Cannot open file: {}", e);
                process::exit(1);
            }
        }
    } else {
        get_sorted_values(BufReader::new(stdin()))
    };

    let stats = StatisticsBundle::from_sorted(&lines, &percents);

    if let Some(v) = stats {
        print!("{}", v.global_stats());
        for s in v.percentile_stats() {
            print!("{}", s)
        }
    } else {
        // use clap error format here?
        let _ = writeln!(stderr(), "No values to compute stats for");
    }
}
