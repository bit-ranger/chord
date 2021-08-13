#!/bin/bash

ssh-add /data/chord/conf/ssh_key.pri
git clone "$chord_git_url" job_repo
cd job_repo || exit
git checkout "$chord_git_branch"
chord-cmd -i"$PWD"/.chord/job -e"$chord_exec_id" -j"$chord_job_name"
