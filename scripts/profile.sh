#!/usr/bin/env bash

set -euo pipefail

cargo build --profile profiling && samply record --no-open ./target/profiling/1brc

# cargo flamegraph --root --profile profiling
