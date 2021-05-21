# 基础镜像 
FROM rust:1.50

# 作者及联系方式   
MAINTAINER bitranger sincerebravefight@gmail.com

WORKDIR /data

CMD ./start.sh
COPY start.sh .
RUN chmod 544 ./start.sh

EXPOSE 9999


COPY .devops/chord/* /data/chord/
COPY . .
RUN cargo build --verbose
RUN cargo test --verbose



