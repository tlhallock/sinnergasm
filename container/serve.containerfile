FROM gcr.io/distroless/static-debian12
COPY ./target/x86_64-unknown-linux-musl/release/serve /app/serve
COPY keys/token.txt /app/keys/token.txt
# TODO: Use a secret manager instead of this
COPY keys/server.key /app/keys/server.key
COPY keys/server.pem /app/keys/server.pem
WORKDIR /app
CMD ["./serve"]
