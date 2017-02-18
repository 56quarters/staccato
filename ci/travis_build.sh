#!/bin/bash -x


main() {
    env
    rustup show
    cargo test --target $TARGET
    cargo test --target $TARGET --bin st
    cargo build --release --target $TARGET
}


main
