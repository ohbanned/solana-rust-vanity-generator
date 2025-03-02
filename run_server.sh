#!/bin/bash
cd "$(dirname "$0")"
echo "Starting Solana Vanity Address Generator Server..."
cargo run --bin solana-vanity-generator
