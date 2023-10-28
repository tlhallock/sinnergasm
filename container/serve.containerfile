FROM gcr.io/distroless/static-debian12
COPY ./target/x86_64-unknown-linux-musl/release/serve /app/serve
COPY ./resources/token.txt /app/resources/token.txt
# TODO: Use a secret manager instead of this
COPY ./resources/server.key /app/resources/server.key
COPY ./resources/ca.crt /app/resources/ca.crt
WORKDIR /app
CMD ["./serve"]
