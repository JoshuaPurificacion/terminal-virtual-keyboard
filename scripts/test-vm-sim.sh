#!/bin/bash
set -e

echo "========================================================"
echo " [VM Simulation] DarkOS Cyberdeck Mode Switch Test      "
echo "========================================================"

cd "$(dirname "$0")/.."

echo "[1/3] Building release binaries (cyberdeck-kb & deck-launcher)..."
cargo build --release

echo "[2/3] Running automated unit & integration tests..."
cargo test -- --nocapture

echo "[3/3] Running mock lifecycle launcher verification..."
./target/release/deck-launcher --test-mock

echo ""
echo "========================================================"
echo " ✅ ALL SIMULATION & LIFECYCLE CHECKS PASSED           "
echo "========================================================"
