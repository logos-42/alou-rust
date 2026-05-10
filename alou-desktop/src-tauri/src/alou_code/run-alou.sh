#!/bin/bash
set -x

echo "=== Step 1: Navigate to rust directory ==="
cd /Users/apple/alou-code/rust
pwd

echo "=== Step 2: Build alou binary ==="
cargo build --release -p rusty-claude-cli
BUILD_EXIT=$?
echo "Build exited with code: $BUILD_EXIT"

echo "=== Step 3: Check if binary exists ==="
if [ -f "target/release/alou" ]; then
    echo "SUCCESS: Binary found"
    ls -lh target/release/alou
    file target/release/alou
else
    echo "ERROR: Binary not found"
    ls -la target/release/ 2>/dev/null || echo "release directory not found"
fi

echo "=== Step 4: Run alou with DeepSeek API ==="
if [ -f "target/release/alou" ]; then
    export ANTHROPIC_API_KEY="sk-0e701b56fd2448b9b8c1b485486a2d23"
    export ANTHROPIC_BASE_URL="https://api.deepseek.com"
    echo "Starting alou with DeepSeek API..."
    ./target/release/alou
else
    echo "Cannot run: binary not found"
    exit 1
fi
