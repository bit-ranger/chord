# chord

[![GitHub Workflow](https://img.shields.io/github/workflow/status/bit-ranger/chord/docker-cargo)](https://github.com/bit-ranger/chord/actions)
[![GitHub Release](https://img.shields.io/github/v/release/bit-ranger/chord?include_prereleases)](https://github.com/bit-ranger/chord/releases/latest)
[![License](https://img.shields.io/github/license/bit-ranger/chord)](https://github.com/bit-ranger/chord/blob/master/LICENSE)

chord - async 并行任务处理框架

### prepare

    cd action/src/action/dubbo/generic-gateway
    mvn package
    cp target/dubbo-generic-gateway-0.0.1-SNAPSHOT.jar /data/chord/bin/dubbo-generic-gateway-0.0.1-SNAPSHOT.jar
    cd ../../../../..

    cd zero/action/dubbo
    mvn package
    java -jar target/dubbo-provider-0.0.1-SNAPSHOT.jar &
    cd ../../..

    docker-compose up -d
    
### run cmd

    cargo run --package  chord-cmd --bin chord-cmd  -- \ 
        -c$(pwd)/zero/devops/chord/conf/application.yml \ 
        -i$(pwd)/zero/devops/chord/job/input \ 
        -o$(pwd)/zero/devops/chord/job/output

### help

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