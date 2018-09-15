#!/bin/sh

set -o xtrace
set -o errexit

PROJECT="tshlabs/staccato"
VERSION=`git describe --abbrev=0 --tags`

docker push "${PROJECT}:latest"
docker push "${PROJECT}:${VERSION}"
