# 基础镜像 
FROM rust:1.53.0

COPY chord-web-worker.sh /usr/bin/chord-web-worker.s
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

ENTRYPOINT ["src/chord-worker.sh"]

RUN cd src \
&& cargo build --release --verbose \
&& cargo test --release --verbose \
&& mv ./target/release/chord-cmd /usr/bin/chord \
&& chmod 755 /usr/bin/chord \
&& cargo clean \
&& rm -rf /usr/local/cargo/registry \
&& cd ..

