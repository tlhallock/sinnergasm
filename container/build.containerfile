FROM rust:latest
RUN apt-get update && apt-get install -y musl-tools protobuf-compiler
RUN rustup target add x86_64-unknown-linux-musl
WORKDIR /build
CMD ["cargo", "build", "--release", "--target", "x86_64-unknown-linux-musl", "--bin", "serve"]
