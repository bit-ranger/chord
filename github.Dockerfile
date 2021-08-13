# 基础镜像 
FROM rust:1.53.0

RUN sudo apt-key adv --keyserver keyserver.ubuntu.com --recv-keys 40976EAF437D05B5
RUN sudo apt-key adv --keyserver keyserver.ubuntu.com --recv-keys 3B4FE6ACC0B21F32
RUN apt-get update
RUN apt-get install -y openjdk-8-jdk

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
&& mv ./target/release/chord-cmd /usr/bin/chord \
&& chmod 755 /usr/bin/chord \
&& cargo clean \
&& rm -rf /usr/local/cargo/registry \
&& cd ..