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
use std::io::{stdin, BufReader};

use clap::{Arg, App, ArgMatches};
use staccato::StatisticsBundle;

const DEFAULT_PERCENTILES: &'static [u8] = &[];


fn parse_cli_opts<'a>(args: Vec<String>) -> ArgMatches<'a> {
    App::new("Staccato")
        .version(crate_version!())
        // TODO: Newlines look bad on the CLI
        .about(
            "Staccato is a program for generating statistics from a stream \
             of numbers from the command line. It reads values from STDIN \
             until the end of the stream (or file being piped in) and computes \
             things about them such as the median, mean, standard deviation, \
             and much more.\
             \n\n\
             It also computes these things for certain portions of the stream. \
             By default it will compute statistics for the entire stream, the \
             lower 75% of values, the lower 90% of values, the lower 95% of \
             values, and the lower 99% of values.\
             \n\n\
             If you've ever used Statsd, the format should seem familiar :)")
        .arg(Arg::with_name("percentiles")
             .short("p")
             .long("percentiles")
             .help(
                 "Comma separated list of percentiles (from 1 to 99, \
                  inclusive) that should have metrics computed. Default \
                  is 75, 90, 95, and 99.")
             .takes_value(true))
        .get_matches_from(args)
}


fn get_percents(pcnt: &str) -> Vec<u8> {
    pcnt.split(",")
        .flat_map(|v| v.parse::<u8>().ok())
        .filter(|&v| v > 0 && v < 100)
        .collect()
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let matches = parse_cli_opts(args);

    // TODO: Handle cases where people want to turn off percentiles
    // TODO: Handle validation of this via clap
    let percents: Vec<u8> = if let Some(p) = matches.value_of("p") {
        get_percents(p)
    } else {
        Vec::from(DEFAULT_PERCENTILES)
    };

    let reader = BufReader::new(stdin());
    let stats = StatisticsBundle::from_reader(reader, &percents);

    print!("{}", stats.global_stats());
    for s in stats.percentile_stats() {
        print!("{}", s)
    }
}
