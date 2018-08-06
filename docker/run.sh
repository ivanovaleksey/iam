#!/usr/bin/env bash
set -ex

run_migrations() {
    docker run --rm \
        --net=host \
        -v "${PWD}:/volume" \
        -w /volume \
        -e DATABASE_URL="${DATABASE_URL}" \
        clux/diesel-cli diesel migration run --migration-dir ${1}
}

DATABASE_URL=${DATABASE_URL:-"postgres://postgres@localhost:5433/postgres"}

docker build -f docker/deps.Dockerfile -t sandbox/iam .
docker run --rm -p 5433:5432 -d sandbox/iam

sleep 3 # wait for postgres to start

run_migrations "abac-rs/migrations"
run_migrations "migrations"
