# 基础镜像 
FROM bitranger/chord:latest

ENTRYPOINT ["chord-web-worker.sh"]

COPY chord-web-worker.sh chord-web-worker.s
COPY chord chord
COPY cmd cmd
COPY flow flow
COPY input input
COPY output output
COPY action action
COPY web web
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock

RUN cargo test --release --verbose \
&& cargo build --release --verbose \
&& mv ./target/release/chord-cmd ./chord-cmd \
&& cargo clean \
&& rm -rf /usr/local/cargo/registry
