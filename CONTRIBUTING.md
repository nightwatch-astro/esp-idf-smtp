# Contributing to esp-idf-smtp

## Getting Started

```bash
git clone https://github.com/nightwatch-astro/esp-idf-smtp.git
cd esp-idf-smtp
cargo build --no-default-features
cargo test --no-default-features
```

## Development

### Building

```bash
cargo build --no-default-features   # host (protocol logic only)
# cargo build                       # ESP32 (requires ESP toolchain)
```

### Testing

All protocol tests run on host without ESP-IDF:

```bash
cargo test --no-default-features    # unit + integration tests
cargo clippy --no-default-features  # lint
cargo fmt --check                   # format check
```

### Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `esp-idf` | Yes | ESP-IDF transport via `esp_tls` |

Without `esp-idf`, only protocol types and `SmtpTransport` trait are available.

## Architecture

```
src/
  config.rs        SmtpConfig, TlsMode, TlsVerify, Credentials
  email.rs         Email builder, Address, header generation
  error.rs         SmtpError typed errors
  protocol.rs      SMTP state machine (platform-agnostic)
  response.rs      SMTP response parser
  transport.rs     SmtpTransport trait
  esp_transport.rs ESP-IDF transport (feature-gated)
  client.rs        SmtpClient (feature-gated)
  mock_transport.rs Mock for testing (cfg(test))

tests/
  response_test.rs  Response parsing tests
  email_test.rs     Email builder tests
  protocol_test.rs  Full protocol session tests
```

## Pull Request Process

1. Create a feature branch from `main`
2. Make your changes with conventional commits (`feat:`, `fix:`, etc.)
3. Ensure `cargo test --no-default-features` passes
4. Ensure `cargo clippy --no-default-features -- -D warnings` is clean
5. Open a PR — CI will run automatically

## License

By contributing, you agree that your contributions will be dual-licensed under MIT and Apache-2.0.
