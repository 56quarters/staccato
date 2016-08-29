#!/bin/bash -x

install_rustup() {
    sh ~/rust/lib/rustlib/uninstall.sh

    curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain=$TRAVIS_RUST_VERSION

    rustc -V
    cargo -V
}


install_target() {
    rustup target add $TARGET || true
}


main() {
    install_rustup
    install_target
}


main
