#!/usr/bin/env bash

set -euo pipefail

cargo build --release && hyperfine --warmup 3 --runs 10 --export-markdown 'benchmark.md' './target/release/1brc'
