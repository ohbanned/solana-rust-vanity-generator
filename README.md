# Solana Vanity Address Generator

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance Rust service for generating vanity Solana wallet addresses with custom prefixes or suffixes. This tool allows you to create memorable Solana public keys that start or end with specific characters of your choice.

## 📋 Features

- Generate Solana addresses with custom prefixes or suffixes
- RESTful API with asynchronous job processing
- Real-time status updates for generation jobs
- Ability to cancel running jobs
- Parallel processing for optimal performance
- Health check endpoint for monitoring

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+ and Cargo

### Running Locally

1. Clone the repository:
```bash
git clone https://github.com/ohbanned/solana-vanity-generator.git
cd solana-vanity-generator
```

2. Build and run:
```bash
cargo build --release
cargo run --release
```

The service will start on `http://127.0.0.1:3001` by default.

### Fork and Customize

This project is designed to be easily forked and modified for your own needs:

1. Fork the repository on GitHub
2. Make your desired changes
3. Run locally using the instructions above

### Environment Variables

- `HOST`: Bind address (default: 127.0.0.1)
- `PORT`: Port to listen on (default: 3001)
- `RUST_LOG`: Log level (default: info)

## 🔧 API Reference

### Generate a Vanity Address

```http
POST /generate
Content-Type: application/json

{
    "pattern": "abc",
    "position": "prefix"  // or "suffix"
}
```

Response:
```json
{
    "job_id": "123e4567-e89b-12d3-a456-426614174000"
}
```

### Check Generation Status

```http
GET /status/:job_id
```

Response (running):
```json
{
    "status": "running"
}
```

Response (complete):
```json
{
    "status": "complete",
    "result": {
        "public_key": "abc...",
        "private_key": "..."
    }
}
```

### Cancel a Running Job

```http
POST /cancel/:job_id
```

Response:
```json
{
    "status": "cancelled"
}
```

### Health Check

```http
GET /health
```

Response:
```json
{
    "status": "ok",
    "timestamp": "2023-09-01T12:34:56Z"
}
```

## 🌟 Examples

The `examples` directory contains sample client code showing how to interact with the API:

- `client.js` - JavaScript example
- `client.py` - Python example

To use the JavaScript example:
```bash
node examples/client.js
```

To use the Python example:
```bash
python examples/client.py
```

These examples demonstrate how to start a generation job, poll for results, and retrieve the generated key pair.

## 🔒 Security Notes

- Private keys are only transmitted once and never stored on the server
- All communication should be over HTTPS in production
- Consider implementing rate limiting for public-facing deployments

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request
