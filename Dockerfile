# 基础镜像 
FROM rust:1.56.0

ENV CARGO_HTTP_MULTIPLEXING false
COPY zero/devops/cargo /usr/local/cargo

COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
COPY chord chord
COPY cmd cmd
COPY flow flow
COPY input input
COPY output output
COPY action action
COPY util util
COPY web web
COPY zero zero

RUN cargo build --release --verbose \
&& cargo test --release --verbose \
&& mv ./target/release/chord-cmd /usr/bin/chord-cmd \
&& chmod 755 /usr/bin/chord-cmd \
&& cargo clean \
&& rm -rf /usr/local/cargo/registry \
&& cd /usr/src