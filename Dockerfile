# 基础镜像 
FROM rust:1.53.0

ENV CARGO_HTTP_MULTIPLEXING false
COPY zero/devops/apt /etc/apt
COPY zero/devops/cargo /usr/local/cargo

RUN apt-key adv --keyserver keyserver.ubuntu.com --recv-keys 40976EAF437D05B5
RUN apt-key adv --keyserver keyserver.ubuntu.com --recv-keys 3B4FE6ACC0B21F32
RUN apt-get update
RUN apt-get install -y openjdk-8-jdk

ENV JAVA_HOME /usr/lib/jvm/java-1.8.0-openjdk-1.8.0.212.b04-0.el7_6.x86_64
ENV PATH $JAVA_HOME/bin:$PATH

COPY chord src/chord
COPY cmd src/cmd
COPY flow src/flow
COPY input src/input
COPY output src/output
COPY action src/action
COPY util src/util
COPY web src/web
COPY Cargo.toml src/Cargo.toml
COPY Cargo.lock src/Cargo.lock


RUN cd src \
&& cargo build --release --verbose \
&& cargo test --release --verbose \
&& mv ./target/release/chord-cmd /usr/bin/chord-cmd \
&& chmod 755 /usr/bin/chord-cmd \
&& cargo clean \
&& rm -rf /usr/local/cargo/registry \
&& cd ..