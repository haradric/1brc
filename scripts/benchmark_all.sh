#!/usr/bin/env bash

set -euo pipefail
set -x

# Settings
BINARIES_DIR="binaries" # Directory to store binaries
RESULTS_FILE="benchmark.md" # File to store benchmark results
HYPERFINE_OPTIONS="--warmup 3 --runs 10 --export-markdown $RESULTS_FILE" # Hyperfine options
PROJECT_NAME="1brc" # Project name

# Create directories and clean up old results
rm -rf "$BINARIES_DIR" "$RESULTS_FILE"
mkdir -p "$BINARIES_DIR"

# Add header to results file
echo "# Benchmark Results" > "$RESULTS_FILE"
echo "" >> "$RESULTS_FILE"

# Counter for numbering binaries
counter=1

# Collect all binaries
git checkout master
for commit in $(git rev-list master --reverse); do
    echo "Processing commit $commit..."
    git checkout "$commit" || { echo "Error switching to commit $commit"; exit 1; }

    # Build the project
    cargo clean
    cargo build --release || { echo "Build failed for commit $commit"; exit 1; }

    # Copy the binary
    BINARY_PATH="target/release/$PROJECT_NAME"
    BINARY_NAME="$BINARIES_DIR/${PROJECT_NAME}_$counter"
    if [[ -f "$BINARY_PATH" ]]; then
        cp "$BINARY_PATH" "$BINARY_NAME"
        echo "Compiled binary saved as $BINARY_NAME"
    else
        echo "Binary not found for commit $commit"
        continue
    fi

    ((counter++))
done

# Run benchmarks after collecting all binaries
echo "Running benchmarks for all binaries..."
binary_list=$(ls "$BINARIES_DIR"/* | sort -n)
hyperfine $HYPERFINE_OPTIONS $binary_list|| {
    echo "Error running benchmarks"
    exit 1
}

# Return to the original branch
git checkout - || echo "Failed to return to the original branch"

echo "All iterations completed. Results saved in $RESULTS_FILE."