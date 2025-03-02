# Solana Vanity Address Generator

A high-performance Rust tool for generating Solana wallet addresses with custom prefixes or suffixes.

## Features

- Generate Solana addresses with custom prefixes or suffixes
- 3-8 character pattern length
- Multiple interfaces:
  - Simple CLI
  - Terminal User Interface (TUI)
  - RESTful API Server

## Quick Start Guide

### Step 1: Start the Server

First, start the server in one terminal:

```bash
./run_server.sh
```

You should see the ASCII art logo and a message that the server is running at `http://127.0.0.1:3001`.

### Step 2: Generate Addresses

#### Option A: Simple CLI (Recommended)

In a new terminal, use the simple CLI command:

```bash
./run_cli.sh abc prefix
```

This will generate a Solana address with "abc" as the prefix.

Or use a suffix instead:

```bash
./run_cli.sh xyz suffix
```

#### Option B: Terminal User Interface (TUI)

If you prefer a graphical interface, run:

```bash
./run_tui.sh
```

Follow the on-screen prompts to set your pattern and position.

#### Option C: Direct API Calls

```bash
# Generate an address with 'abc' prefix
curl -X POST http://127.0.0.1:3001/generate -H "Content-Type: application/json" -d '{"pattern":"abc","position":"prefix"}'

# Check status using the job_id from the response
curl http://127.0.0.1:3001/status/<job_id>
```

## Troubleshooting

- **"Address already in use" error**: The server is already running in another terminal. Either use that instance or stop it and start again.
- **Blue screen in TUI**: Try resizing your terminal window or use the simple CLI option instead.
- **Connection errors**: Make sure the server is running before using the client tools.

## Security Notes

- Private keys are transmitted only once when the address is found
- No persistent storage of sensitive information
- Always securely store your private keys after generation

## Build from Source

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/ohbanned/solana-vanity-generator.git
cd solana-vanity-generator
cargo build --release

# Run
./run_server.sh  # In one terminal
./run_cli.sh abc prefix  # In another terminal
```

## License

MIT

Built by Ban (Github: @ohbanned)
