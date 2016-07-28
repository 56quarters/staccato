# Staccato

[![Build Status](https://travis-ci.org/tshlabs/staccato.svg?branch=master)](https://travis-ci.org/tshlabs/staccato)
[![crates.io](https://img.shields.io/crates/v/staccato.svg)](https://crates.io/crates/staccato/)

Statistics from the command line!

Staccato is a command line program that lets you compute statistics from
numbers piped in via STDIN. It computes things about the stream of numbers
like min, max, mean, median, and standard deviation. It can also compute
these things about some subset of the stream, for example the lower 95% of
values.

## Features

Lots of em! Better examples coming soon!

## Install

For now, the only way to install Staccato is from source, using Cargo:

```
git clone https://github.com/tshlabs/staccato.git && cd staccato
cargo install
st --help
```

Better installation and docs coming soon!

## Examples

Some examples of how to use Staccato are given below. Note that these
examples assume you are familiar with standard Unix command line tools
like `awk`, `cut`, and `tail`.

### File of Values

The most obvious use case for Staccato is when you already have a file
full of numbers and you want to know things about them. For example, imagine
you have a file called `timings.log` that looks like this:

```
0.572124
0.623724
1.043369
0.563586
1.603538
0.540765
1.677319
0.170808
0.147564
```

To get statistics about those values, you'd run Staccato like this:

```
$ st < timings.log
count: 9
sum: 6.942797
mean: 0.7714218888888889
upper: 1.677319
lower: 0.147564
median: 0.572124
stddev: 0.5265074031965414
```

### Application Log File

Another good use of Staccato is to compute the statistics from some
particular field or value being written to a log file. Imagine that
you have an access log called `access.log` for your web application
that looks something like the following:

```
2016-08-29T02:14:32 GET /some-url-path/?foo=bar 200 3.84639
```

... where the fields in this log represent:

```
$TIMESTAMP $HTTP_METHOD $REQUEST_URL $HTTP_RESPONSE $RESPONSE_TIME_IN_MS
```

To get statistics about the most recent 100 response times for your
application, you might use Staccato like this:

```
$ tail -n 100 /var/log/my-application/access.log | cut -d ' ' -f 5 | st
count: 100
mean: 0.20346768999999995
upper: 3.84639
lower: 0.005766
median: 0.021009
stddev: 0.6087083786414262
```

## Documentation

Coming soon!

## Source

The source code is available on GitHub at https://github.com/tshlabs/staccato

## Changes

Release notes for Staccato can be found in the [CHANGES.md](CHANGES.md) file.

## Development

Staccato uses Cargo for performing various development tasks.

To build Staccato:

```
$ cargo build
```

To run tests:

```
$ cargo test
```

To run benchmarks:

```
$ cargo bench
```

To build documentation:

```
$ cargo doc
```

## License

Staccato is available under the terms of the [GPL, version 3](LICENSE).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be licensed as above, without any
additional terms or conditions.
