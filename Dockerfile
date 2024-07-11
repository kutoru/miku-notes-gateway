FROM rust:1.75

RUN apt-get update && DEBIAN_FRONTEND=nointeractive apt-get install --no-install-recommends --assume-yes protobuf-compiler

WORKDIR /usr/src/miku-notes-gateway
COPY . .
RUN cargo install --path .

CMD ["miku-notes-gateway"]
