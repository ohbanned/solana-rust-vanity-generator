#!/bin/bash
cd "$(dirname "$0")"
echo "Starting Solana Vanity Address Generator TUI..."
export TERM=xterm-256color
cargo run --bin vanity-tui
