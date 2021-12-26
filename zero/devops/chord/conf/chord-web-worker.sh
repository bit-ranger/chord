#!/bin/bash

git clone "$chord_git_url" job_repo
cd job_repo || exit
git checkout "$chord_git_branch"
ls -la "$PWD"/.chord/src
npm install --prefix .chord
npm run build --prefix .chord
export RUST_BACKTRACE=full
chord run -i"$PWD"/.chord/src -e"$chord_exec_id" -j"$chord_job_name"