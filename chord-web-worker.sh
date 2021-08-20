#!/bin/bash                                                                                                                                                                                             2
cp -r /data/chord/conf/.ssh /root/
cp -r /data/chord/conf/.keys /root/
git clone "$chord_git_url" job_repo
cd job_repo || exit
git checkout "$chord_git_branch"
ls -la "$PWD"/.chord/job
export RUST_BACKTRACE=full
chord-cmd -i"$PWD"/.chord/job -e"$chord_exec_id" -j"$chord_job_name"