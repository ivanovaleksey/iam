# Build
FROM clux/muslrust as build-stage

WORKDIR "/build"
COPY . .
RUN cargo build --bin cli --release

# Package
FROM alpine

RUN mkdir /app
WORKDIR /app

COPY --from=build-stage /build/target/release/cli ./iam-cli

ENTRYPOINT ["/app/iam-cli"]
