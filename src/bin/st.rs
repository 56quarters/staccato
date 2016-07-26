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
extern crate getopts;

use std::env;
use std::cmp::Ordering;
use std::fmt;
use std::io::{stdin, stderr, BufRead, BufReader, Write};
use std::process;

use getopts::Options;

use staccato::{Statistics};

const DEFAULT_PERCENTILES: &'static [u8] = &[75, 90, 95, 99];


fn get_values<T: BufRead>(reader: T) -> Vec<f64> {
    let vals: Vec<f64> = reader.lines()
        .flat_map(|v| v.ok())
        .map(|v| v.parse::<f64>().ok())
        .filter_map(|v| v)
        .collect();

    vals
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

    // TODO: Do this better
    let percents: Vec<u8> = if let Some(p) = matches.opt_str("p") {
        get_percents(p)
    } else {
        Vec::from(DEFAULT_PERCENTILES)
    };

    let reader = BufReader::new(stdin());
    let mut values = get_values(reader);
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Less));

    let global_stats = Statistics::from(&values, None);
    print!("{}", global_stats);

    for p in percents {
        print!("{}", Statistics::from(&values, Some(p as u8)));
    }
}
