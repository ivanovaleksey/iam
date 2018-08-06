FROM clux/diesel-cli

WORKDIR /root

COPY ./abac-rs/migrations ./abac-rs/migrations/
COPY ./migrations ./migrations
# Diesel CLI 1.3.1 requires Cargo.toml to find project root
COPY ./Cargo.toml ./Cargo.toml

COPY ./scripts/run-migrations.sh ./run-migrations.sh

ENTRYPOINT ["./run-migrations.sh"]
