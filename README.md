# chord

[![GitHub Workflow](https://img.shields.io/github/workflow/status/bit-ranger/chord/docker-cargo)](https://github.com/bit-ranger/chord/actions)
[![GitHub Release](https://img.shields.io/github/v/release/bit-ranger/chord?include_prereleases)](https://github.com/bit-ranger/chord/releases/latest)
[![License](https://img.shields.io/github/license/bit-ranger/chord)](https://github.com/bit-ranger/chord/blob/master/LICENSE)

chord - async 并行任务处理框架



## usage

### cmd

    cargo run --package  chord-cmd --bin chord-cmd  -- \ 
        -c$(readlink -f .)/zero/devops/chord/conf/application.yml \ 
        -i$(readlink -f .)/zero/devops/chord/job/input \ 
        -o$(readlink -f .)/zero/devops/chord/job/output


#### help
    cargo run --package  chord-cmd --bin chord-cmd -- --help

##### 

    chord 0.1.0

    USAGE:
    chord-cmd [OPTIONS] --input <input> --output <output>

    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information

    OPTIONS:
        -c, --config <config>    config file path [default: /data/chord/conf/application.yml]
        -i, --input <input>      input dir
        -o, --output <output>    output dir
    
### example
[example](https://github.com/bit-ranger/chord/tree/master/zero/devops/chord/job/input)