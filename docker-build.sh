#!/bin/sh

set -o xtrace
set -o errexit

PROJECT="tshlabs/staccato"
VERSION=`git describe --abbrev=0 --tags`

git checkout "$VERSION"
docker build -t "${PROJECT}:latest" .
docker tag "${PROJECT}:latest" "${PROJECT}:${VERSION}"
git checkout -

docker run -ti --rm "${PROJECT}:latest"
