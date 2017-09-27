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

extern crate staccato;
#[macro_use]
extern crate clap;

use std::env;
use std::fs::File;
use std::io::{stdin, BufReader, Write};
use std::process;

use clap::{Arg, App, ArgMatches};
use staccato::{
    get_values,
    SortingPolicy,
    StatisticsBundle,
    StatisticsFormatter,
    KeyValueSep,
};


const DEFAULT_PERCENTILES: &'static [u8] = &[];
const DEFAULT_SEPARATOR: KeyValueSep = KeyValueSep::Colon;
const DEFAULT_TERM_WIDTH: usize = 72;


macro_rules! eprintln {
    ($($arg:tt)*) => {{
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed to write to stderr");
    }}
}


fn parse_cli_opts<'a>(args: Vec<String>) -> ArgMatches<'a> {
    App::new("Staccato")
        .version(crate_version!())
        .about("\nStatistics from the command line!")
        .set_term_width(DEFAULT_TERM_WIDTH)
        .after_help(concat!(
            "Staccato is a program for generating statistics from a stream ",
            "of numbers from the command line. It reads values from a file or ",
            "standard input until the end of the stream (or file) and computes ",
            "things about them such as the median, mean, standard deviation, ",
            "and much more.",
            "\n\n",
            "By default it will compute statistics for the entire stream. You ",
            "can have it additionally compute statistics for some subset of the ",
            "values of the stream. For example, using the argument `-p 25,50` ",
            "would compute statistics for the lower 25% of values and lower 50% ",
            "of values."))
        .arg(Arg::with_name("percentiles")
             .short("p")
             .long("percentiles")
             .help(concat!(
                 "Comma separated list of percentiles (from 1 to 99, ",
                  "inclusive) that should have metrics computed. Default ",
                  "is not to compute metrics for any specific percentiles, ",
                  "only the global metrics."))
             .takes_value(true)
             .validator(validate_percents))
        .arg(Arg::with_name("separator")
             .short("s")
             .long("separator")
             .help(concat!(
                 "Type of separator to use when printing keys and values. ",
                 "Possible values for this option are the literal string ",
                 "'tab' for the tab character, the literal string 'colon' ",
                 "for a colon and space, or any other string to use that ",
                 "as a separator. For example you could use the string ",
                 "' => ' as a separator. Default is to use a colon and a ",
                 "space."))
             .takes_value(true))
        // Note that we aren't using any validators for the file input.
        // We'll just try to open it and see what happens. Otherwise we
        // become susceptible to race conditions.
        .arg(Arg::with_name("file")
             .help(concat!(
                 "Optional file to read values to from. If not supplied ",
                 "values will be read from standard input. The values are ",
                 "expected to be floating point or integer values, one per ",
                 "line. Leading or trailing whitespace will be removed before ",
                 "parsing each value."))
             .index(1))
        .get_matches_from(args)
}


fn validate_percents(v: String) -> Result<(), String> {
    for p in v.split(',') {
        match p.parse::<u8>() {
            Ok(i) if i > 0 && i < 100 => i,
            _ => {
                return Err("Invalid percentile value".to_string());
            }
        };
    }

    Ok(())
}


fn parse_percents<'a>(matches: &ArgMatches<'a>) -> Vec<u8> {
    // This is pretty blunt: if it's not a valid value we just drop it
    // and move on. Shouldn't actually matter in practice since we've
    // already validated the value via the `validate_percents` method.
    // If it's present here, it should be valid, if it's not present
    // we use the defaults.
    if let Some(p) = matches.value_of("percentiles") {
        p.split(',')
            .flat_map(|v| v.parse::<u8>())
            .filter(|&v| v >= 1 && v <= 99)
            .collect()
    } else {
        Vec::from(DEFAULT_PERCENTILES)
    }
}


fn parse_separator<'a>(matches: &ArgMatches<'a>) -> KeyValueSep {
    value_t!(matches, "separator", KeyValueSep).unwrap_or_else(|_| {
        DEFAULT_SEPARATOR.clone()
    })
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let matches = parse_cli_opts(args);
    let percents = parse_percents(&matches);
    let separator = parse_separator(&matches);
    let sorting = if !percents.is_empty() {
        SortingPolicy::Sorted
    } else {
        SortingPolicy::Unsorted
    };

    let line_result = if let Some(f) = matches.value_of("file") {
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

    let stats = StatisticsBundle::with_percentiles(&lines, &percents);
    if let Some(v) = stats {
        print!("{}", StatisticsFormatter::with_sep(&v, separator));
    } else {
        eprintln!("warning: No values to compute stats for");
    }
}


#[cfg(test)]
mod tests {
    use super::{
        parse_percents,
        parse_separator,
        validate_percents,
        DEFAULT_PERCENTILES,
        DEFAULT_SEPARATOR
    };
    use staccato::KeyValueSep;
    use clap::{Arg, App};

    #[test]
    fn test_validate_percents_err_not_in_range() {
        let percents = "75,90,100,110".to_string();
        let res = validate_percents(percents);

        assert!(res.is_err());
    }

    #[test]
    fn test_validate_percents_lower_bound() {
        let percents = "0,50,75".to_string();
        let res = validate_percents(percents);

        assert!(res.is_err());
    }

    #[test]
    fn test_validate_percents_upper_bound() {
        let percents = "50,75,100".to_string();
        let res = validate_percents(percents);

        assert!(res.is_err());
    }

    #[test]
    fn test_validate_percents_err_not_a_number() {
        let percents = "75,banana".to_string();
        let res = validate_percents(percents);

        assert!(res.is_err());
    }

    #[test]
    fn test_validate_percents_ok() {
        let percents = "75,90,95,98".to_string();
        let res = validate_percents(percents);

        assert!(res.is_ok());
    }

    #[test]
    fn test_parse_percents_no_matches() {
        let args = App::new("test")
            .arg(Arg::with_name("percentiles")
                 .short("p")
                 .long("percentiles")
                 .takes_value(true))
            .get_matches_from(vec!["test"]);

        let expected: Vec<u8> = Vec::from(DEFAULT_PERCENTILES);
        let parsed = parse_percents(&args);

        assert_eq!(expected, parsed);
    }

    #[test]
    fn test_parse_percents_matches() {
        let args = App::new("test")
            .arg(Arg::with_name("percentiles")
                 .short("p")
                 .long("percentiles")
                 .takes_value(true))
            .get_matches_from(vec!["test", "-p", "90,95"]);

        let expected: Vec<u8> = vec![90, 95];
        let parsed = parse_percents(&args);

        assert_eq!(expected, parsed);
    }

    #[test]
    fn test_parse_separator_no_matches() {
        let args = App::new("test")
            .arg(Arg::with_name("separator")
                 .short("s")
                 .long("separator")
                 .takes_value(true))
            .get_matches_from(vec!["test"]);

        let expected = DEFAULT_SEPARATOR;
        let parsed = parse_separator(&args);

        assert_eq!(expected, parsed);
    }

    #[test]
    fn test_parse_separator_matches_tab() {
        let args = App::new("test")
            .arg(Arg::with_name("separator")
                 .short("s")
                 .long("separator")
                 .takes_value(true))
            .get_matches_from(vec!["test", "-s", "tab"]);

        let expected = KeyValueSep::Tab;
        let parsed = parse_separator(&args);

        assert_eq!(expected, parsed);
    }

    #[test]
    fn test_parse_separator_matches_other() {
        let args = App::new("test")
            .arg(Arg::with_name("separator")
                 .short("s")
                 .long("separator")
                 .takes_value(true))
            .get_matches_from(vec!["test", "-s", " = "]);

        let expected = KeyValueSep::Other(" = ".to_string());
        let parsed = parse_separator(&args);

        assert_eq!(expected, parsed);
    }
  }
