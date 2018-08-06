FROM alpine

RUN mkdir /app
WORKDIR /app

COPY ./target/x86_64-unknown-linux-musl/release/iam ./iam
COPY ./migrations ./migrations/

ENTRYPOINT ["/app/iam"]
