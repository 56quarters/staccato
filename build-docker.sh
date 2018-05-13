#!/bin/sh

set -o xtrace
set -o errexit

PROJECT="tshlabs/staccato"
VERSION=`git describe --abbrev=0 --tags`
git checkout "$VERSION"
cargo build --release --target=x86_64-unknown-linux-musl
strip target/x86_64-unknown-linux-musl/release/st
docker build -t "${PROJECT}:latest" .
docker tag "${PROJECT}:latest" "${PROJECT}:${VERSION}"
docker push "${PROJECT}:latest"
docker push "${PROJECT}:${VERSION}"
git checkout -
