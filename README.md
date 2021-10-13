# chord

[![GitHub Workflow](https://img.shields.io/github/workflow/status/bit-ranger/chord/docker-cargo)](https://github.com/bit-ranger/chord/actions)
[![GitHub Release](https://img.shields.io/github/v/release/bit-ranger/chord?include_prereleases)](https://github.com/bit-ranger/chord/releases/latest)
[![License](https://img.shields.io/github/license/bit-ranger/chord)](https://github.com/bit-ranger/chord/blob/master/LICENSE)

chord - async parallel task executor

可用于自动化测试

### run cmd

    cargo build --release

    target/release/chord-cmd run -i$(pwd)/.chord/job -techo

### help

    target/release/chord-cmd run --help

### rest api testing

[example](https://github.com/bit-ranger/chord/tree/master/.chord/job/restapi)

### example

[example](https://github.com/bit-ranger/chord/tree/master/.chord/job)

### workflow

[github action](https://github.com/bit-ranger/chord/blob/master/.github/workflows/dev.yml)