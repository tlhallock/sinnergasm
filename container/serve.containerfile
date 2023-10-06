FROM gcr.io/distroless/cc-debian10
COPY ./target/x86_64-unknown-linux-musl/release/serve /app/serve
WORKDIR /app
CMD ["./serve"]
