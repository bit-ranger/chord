# chord

[![GitHub Workflow](https://img.shields.io/github/workflow/status/bit-ranger/chord/master?label=master)](https://github.com/bit-ranger/chord/actions/workflows/master.yml)
[![GitHub Release](https://img.shields.io/github/v/release/bit-ranger/chord?include_prereleases)](https://github.com/bit-ranger/chord/releases/latest)
[![License](https://img.shields.io/github/license/bit-ranger/chord)](https://github.com/bit-ranger/chord/blob/master/LICENSE)

chord - async parallel task executor

可用于自动化测试

### download

    https://github.com/bit-ranger/chord/releases

### run command

    chord run -i$(pwd)/.chord/job/echo

### run command from source code

    git clone https://github.com/bit-ranger/chord.git
    
    cargo build --release
    
    target/release/chord run -i$(pwd)/.chord/job/echo

### help

    target/release/chord run --help

### rest api testing

[restapi](https://github.com/bit-ranger/chord/tree/master/.chord/job/restapi)

### example

[example](https://github.com/bit-ranger/chord/tree/master/.chord/job)

### workflow

[github action](https://github.com/bit-ranger/chord/blob/master/.github/workflows/master.yml)