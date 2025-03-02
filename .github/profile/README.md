# Solana Vanity Generator

Generate custom Solana wallet addresses with specific prefixes or suffixes.

## What is this?

This tool allows you to create Solana wallet addresses that begin or end with specific characters, making them more recognizable and personalized.

## Key Features

- ⚡ High performance Rust implementation
- 🔄 Asynchronous generation with status updates
- 🌐 Simple HTTP API
- 🔍 Support for both prefix and suffix patterns

## Getting Started

Check out our [main README](../../README.md) for installation and usage instructions.

## Quick API Example

```bash
# Start a generation job
curl -X POST http://localhost:3001/generate \
  -H "Content-Type: application/json" \
  -d '{"pattern":"abc","position":"prefix"}'

# Check job status
curl http://localhost:3001/status/YOUR_JOB_ID
```
