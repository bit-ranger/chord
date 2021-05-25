# 基础镜像 
FROM rust:1.50

# 作者及联系方式   
MAINTAINER bitranger sincerebravefight@gmail.com

WORKDIR /data

EXPOSE 9999

ENV CARGO_HTTP_MULTIPLEXING false
COPY .local/apt /etc/apt
COPY .local/cargo /usr/local/cargo
COPY .local/chord /data/chord
COPY . .
RUN cargo test --verbose \
&& cargo build --release --verbose \
&& mv ./target/release/chord-web ./chord-web \
&& cargo clean \
&& rm -rf /usr/local/cargo/registry


