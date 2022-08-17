#!/bin/bash

set -e

entry="$(pwd)"

members=("core" "util" "action" "input" "output" "flow" "cli" "web")

echo "check members"
for mod in "${members[@]}"
do
  cd "$mod"
  echo "entering $(pwd)"
  cargo check --all-features
  cd "$entry"
done

echo "publish members"
for mod in "${members[@]}"
do
  cd "$mod"
  echo "entering $(pwd)"
  cargo publish
  cd "$entry"
  sleep 1m
done
