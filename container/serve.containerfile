FROM gcr.io/distroless/static-debian12
COPY ./target/x86_64-unknown-linux-musl/release/serve /app/serve
COPY ./token.txt /app/token.txt
WORKDIR /app
CMD ["./serve"]
