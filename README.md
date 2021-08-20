# chord

[![GitHub Workflow](https://img.shields.io/github/workflow/status/bit-ranger/chord/docker-cargo)](https://github.com/bit-ranger/chord/actions)
[![GitHub Release](https://img.shields.io/github/v/release/bit-ranger/chord?include_prereleases)](https://github.com/bit-ranger/chord/releases/latest)
[![License](https://img.shields.io/github/license/bit-ranger/chord)](https://github.com/bit-ranger/chord/blob/master/LICENSE)

chord - async 并行任务处理框架


    
### run cmd

    mkdir -p /data/chord/job/output

    cargo run --package  chord-cmd --bin chord-cmd  -- \ 
        -c$(pwd)/zero/devops/chord/conf/cmd.yml \ 
        -i$(pwd)/.chord/job \
        -techo


### other action

    docker-compose up -d

#### dubbo
    cd action/src/action/dubbo/generic-gateway
    mvn package
    cp target/dubbo-generic-gateway-0.0.1-SNAPSHOT.jar /data/chord/bin/dubbo-generic-gateway-0.0.1-SNAPSHOT.jar
    cd ../../../../..

    cd zero/action/dubbo
    mvn package
    java -jar target/dubbo-provider-0.0.1-SNAPSHOT.jar &
    cd ../../..

    

### help

    cargo run --package  chord-cmd --bin chord-cmd -- --help


### example

[example](https://github.com/bit-ranger/chord/tree/master/.chord/job)

### workflow

[github action](https://github.com/bit-ranger/chord/blob/master/.github/workflows/multi.yml)