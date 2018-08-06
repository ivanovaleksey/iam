#!/usr/bin/env sh
set -ex

diesel migration run --migration-dir abac-rs/migrations/
diesel migration run --migration-dir migrations/
