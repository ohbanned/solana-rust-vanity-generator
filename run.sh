#!/bin/bash

# Build and run the Solana Vanity Address Generator

set -e

echo "Building Solana Vanity Address Generator..."
cargo build --release

echo "Starting the service..."
RUST_LOG=info ./target/release/solana-vanity-generator
