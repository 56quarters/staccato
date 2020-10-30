# Staccato

[![Build Status](https://travis-ci.org/56quarters/staccato.svg?branch=master)](https://travis-ci.org/56quarters/staccato)
[![crates.io](https://img.shields.io/crates/v/staccato.svg)](https://crates.io/crates/staccato/)

Statistics from the command line!

Staccato (`st` for short) is a command line program that lets you compute
statistics from values from a file or standard input. It computes things
about the stream of numbers like min, max, mean, median, and standard
deviation. It can also compute these things about some subset of the stream,
for example the lower 95% of values.

## Install

### Cargo (Rust build tool)

Staccato is a Rust project. If you want to install it, you'll need the Rust
toolchain. For more information about how to install Rust see https://www.rustup.rs/

After you have Rust installed, you can use Cargo to install Staccato.

```
cargo install --force staccato
st --help
```

### Docker

Docker images of Staccato are pushed to Docker Hub for each release. To run the latest
version, use the following command.

```
docker run --rm --tty --interactive tshlabs/staccato:latest
```

## Examples

Some examples of how to use Staccato are given below. Note that these
examples assume you are familiar with standard Unix command line tools
like `awk`, `cut`, and `tail`.

### File of Values

The most obvious use case for Staccato is when you already have a file
full of numbers and you want to know things about them. For example, imagine
you have a file called `timings.log` like so:

```
$ cat << EOF > timings.log
0.572124
0.623724
1.043369
0.563586
1.603538
0.540765
1.677319
0.170808
0.147564
EOF
```

To get statistics about those values, you'd run Staccato like this:

```
$ st timings.log
count: 9
sum: 6.94279
mean: 0.77142
upper: 1.67731
lower: 0.14756
median: 0.57212
stddev: 0.52650
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
mean: 0.20346
upper: 3.84639
lower: 0.00577
median: 0.02101
stddev: 0.60871
```

## Source

The source code is available on GitHub at https://github.com/56quarters/staccato

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
