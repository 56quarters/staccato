#!/bin/bash -x

install_rustup() {
    sh ~/rust/lib/rustlib/uninstall.sh

    curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain=$TRAVIS_RUST_VERSION

    rustc -V
    cargo -V
}


install_target() {
    local default_target=`rustup target list | grep '(default)' | cut -d ' ' -f 1`
    if [[ "${default_target}" != "${TARGET}" ]]; then
        rustup target add $TARGET
    fi
}


main() {
    install_rustup
    install_target
}


main