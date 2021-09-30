#!/bin/bash

git clone "$chord_git_url" job_repo
cd job_repo || exit
git checkout "$chord_git_branch"
ls -la "$PWD"/.chord/job
export RUST_BACKTRACE=full
chord-cmd run -i"$PWD"/.chord/job -e"$chord_exec_id" -j"$chord_job_name"