# 基础镜像 
FROM rust:1.53.0

# 作者及联系方式   
MAINTAINER bit-ranger sincerebravefight@gmail.com

WORKDIR /workdir

EXPOSE 9999

ENV CARGO_HTTP_MULTIPLEXING false
COPY zero/devops/apt /etc/apt
COPY zero/devops/cargo /usr/local/cargo

RUN apt-get install software-properties-common
RUN add-apt-repository ppa:deadsnakes/ppa
RUN apt-get update
RUN apt-get install python3.7 -y

COPY chord chord
COPY cmd cmd
COPY flow flow
COPY input input
COPY output output
COPY action action
COPY web web
COPY Cargo.toml Cargo.toml

RUN cargo build --release --verbose \
&& cargo test --release --verbose \
&& mv ./target/release/chord-web ./chord-web \
&& mv ./target/release/chord-cmd ./chord-cmd \
&& cargo clean \
&& rm -rf /usr/local/cargo/registry
