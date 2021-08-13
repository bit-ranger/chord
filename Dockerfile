# 基础镜像 
FROM rust:1.53.0

ENV CARGO_HTTP_MULTIPLEXING false
COPY zero/devops/apt /etc/apt
COPY zero/devops/cargo /usr/local/cargo



COPY chord-web-worker.sh /usr/bin/chord-web-worker.sh
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
&& chmod 755 /usr/bin/chord \
&& chmod 755 /usr/bin/chord-web-worker.sh \
&& cargo clean \
&& rm -rf /usr/local/cargo/registry \
&& cd ..

