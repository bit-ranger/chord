# 基础镜像 
FROM bitranger/chord:latest

ENV CARGO_HTTP_MULTIPLEXING false
COPY zero/devops/apt /etc/apt
COPY zero/devops/cargo /usr/local/cargo

ENTRYPOINT ["chord-web-worker.sh"]

COPY chord-web-worker.sh chord-web-worker.sh
COPY chord chord
COPY cmd cmd
COPY flow flow
COPY input input
COPY output output
COPY action action
COPY util util
COPY web web
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock

RUN cargo build --release --verbose \
&& cargo test --release --verbose \
&& mv ./target/release/chord-cmd ./chord-cmd \
&& cargo clean \
&& rm -rf /usr/local/cargo/registry
