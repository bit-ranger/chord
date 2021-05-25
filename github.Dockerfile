# 基础镜像 
FROM rust:1.50

# 作者及联系方式   
MAINTAINER bitranger sincerebravefight@gmail.com

WORKDIR /data

EXPOSE 9999

COPY .github/chord /data/chord
COPY . .
RUN cargo build --verbose
RUN cargo test --verbose
RUN mv ./target/release/chord-web ./chord-web
RUN cargo clean
RUN rm -rf /usr/local/cargo/registry


