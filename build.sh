#!/bin/bash

set -e

entry="$(pwd)"

members=("core" "util" "action" "input" "output" "flow" "cli" "web")

echo "build members"
for mod in "${members[@]}"
do
  cd "$mod"
  echo "entering $(pwd)"
  cargo build --all-features --verbose --release
  cd "$entry"
done
cargo build --all-features --verbose --release


echo "test members"
for mod in "${members[@]}"
do
  cd "$mod"
  echo "entering $(pwd)"
  cargo test --all-features --verbose --release
  cd "$entry"
done
cargo test --all-features --verbose --release
