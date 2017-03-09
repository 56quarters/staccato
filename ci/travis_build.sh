#!/bin/bash -x


main() {
    env
    rustup show
    cargo test --target $TARGET
    #cargo build --release --target $TARGET
}


main
