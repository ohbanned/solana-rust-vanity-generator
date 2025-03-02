#!/bin/bash
cd "$(dirname "$0")"
echo "Solana Vanity Address Generator CLI"
echo "-----------------------------------"
cargo run --bin vanity "$@"
