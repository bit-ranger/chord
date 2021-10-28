# 基础镜像 
FROM rust:1.56.0

ENV CARGO_HTTP_MULTIPLEXING false
COPY zero/devops/apt /etc/apt
COPY zero/devops/cargo /usr/local/cargo

RUN apt-key adv --keyserver keyserver.ubuntu.com --recv-keys 40976EAF437D05B5 \
&& apt-key adv --keyserver keyserver.ubuntu.com --recv-keys 3B4FE6ACC0B21F32 \
&& apt-get update -y \
&& apt-get install -y openjdk-8-jdk

ENV JAVA_HOME /usr/lib/jvm/java-8-openjdk-amd64
ENV PATH $JAVA_HOME/bin:$PATH

RUN apt-get install -y maven

WORKDIR /usr/src

COPY action/src/action/dubbo/generic-gateway action/src/action/dubbo/generic-gateway

RUN cd action/src/action/dubbo/generic-gateway \
&& mvn package \
&& mkdir -p /root/.chord/lib \
&& cp target/dubbo-generic-gateway-0.0.1-SNAPSHOT.jar /root/.chord/lib/dubbo-generic-gateway-0.0.1-SNAPSHOT.jar \
&& cd /usr/src

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