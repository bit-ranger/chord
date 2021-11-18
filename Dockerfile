# 基础镜像 
FROM rust:1.56.0

WORKDIR /usr/src


COPY zero/devops/apt/sources.list /etc/apt/sources.list
RUN apt-get update -y


# maven
RUN apt-get install -y maven
COPY zero/devops/maven/settings.xml /root/.m2/settings.xml


# dubbo generic gateway
COPY action/src/action/dubbo/generic-gateway action/src/action/dubbo/generic-gateway
RUN cd action/src/action/dubbo/generic-gateway \
&& mvn package \
&& mkdir -p /root/.chord/lib \
&& cp target/dubbo-generic-gateway-0.0.1-SNAPSHOT.jar /root/.chord/lib/dubbo-generic-gateway-0.0.1-SNAPSHOT.jar \
&& mvn clean \
&& rm -rf /root/.m2 \
&& cd /usr/src


# cargo
ENV CARGO_HTTP_MULTIPLEXING false
COPY zero/devops/cargo/config /usr/local/cargo/config


# chord
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
COPY chord chord
COPY cli cli
COPY flow flow
COPY input input
COPY output output
COPY action action
COPY util util
COPY web web
COPY zero zero
RUN cargo build --release --verbose \
&& cargo test --release --verbose \
&& mv ./target/release/chord /usr/bin/chord \
&& chmod 755 /usr/bin/chord \
&& cargo clean \
&& rm -rf /usr/local/cargo/registry \
&& cd /usr/src