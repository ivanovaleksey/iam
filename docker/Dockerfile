# Build
FROM clux/muslrust as build-stage

WORKDIR "/build"
COPY . .
RUN cargo build --bin iam --release

# Package
FROM alpine

RUN mkdir /app
WORKDIR /app

COPY --from=build-stage /build/target/release/iam .
COPY ./migrations ./migrations/

ENTRYPOINT ["/app/iam"]
