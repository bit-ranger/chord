ssh-add /data/chord/conf/ssh_key.pri
git clone "$(chord.git_url)" job_repo
cd job_repo
git checkout "$(chord.git_branch)"
chord-cmd -i.chord/job
