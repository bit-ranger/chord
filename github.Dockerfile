# 基础镜像 
FROM rust:1.52.1

# 作者及联系方式   
MAINTAINER bitranger sincerebravefight@gmail.com

WORKDIR /data

EXPOSE 9999

COPY zero/devops/chord /data/chord
COPY . .
RUN cargo build --release --verbose \
&& cargo test --release --verbose \
&& mv ./target/release/chord-web ./chord-web \
&& mv ./target/release/chord-cmd ./chord-cmd \
&& cargo clean \
&& rm -rf /usr/local/cargo/registry

