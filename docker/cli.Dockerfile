FROM alpine

RUN mkdir /app
WORKDIR /app

COPY ./target/x86_64-unknown-linux-musl/release/cli ./iam-cli

ENTRYPOINT ["/app/iam-cli"]
