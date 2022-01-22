#!/bin/bash

set -e

entry="$(pwd)"

for mod in "core" "util" "action" "input" "output" "flow" "cli" "web"
do
  cd "$mod"
  echo "entering $(pwd)"
  cargo check
  cargo publish
  cd "$entry"
  sleep 1m
done