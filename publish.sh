#!/bin/bash

set -e

cd core
cargo check
cargo publish

sleep 1m
cd ..
cd util
cargo check
cargo publish

sleep 1m
cd ..
cd action
cargo check
cargo publish

sleep 1m
cd ..
cd input
cargo check
cargo publish

sleep 1m
cd ..
cd output
cargo check
cargo publish

sleep 1m
cd ..
cd flow
cargo check
cargo publish

sleep 1m
cd ..
cd cli
cargo check
cargo publish

sleep 1m
cd ..
cd web
cargo check
cargo publish
