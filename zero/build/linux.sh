#!/bin/bash

# linux
docker run --rm -u "$(id -u)":"$(id -g)" -v "$(pwd)":/workdir multiarch/crossbuild /usr/local/rust/bin/cargo build --release --verbose