# esp-idf-smtp

Lightweight SMTP client for ESP32 devices using [esp_tls](https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/protocols/esp_tls.html).

Supports implicit TLS, STARTTLS, and plaintext connections with AUTH PLAIN/LOGIN authentication. Builder pattern for email composition. Host-testable protocol logic via transport trait abstraction.

## Features

- **Three TLS modes**: Implicit TLS (port 465), STARTTLS (port 587), Plain (port 25)
- **Authentication**: AUTH PLAIN and AUTH LOGIN, auto-selected from server capabilities
- **Multiple recipients**: To, CC, BCC with correct header generation
- **RFC compliant**: Dot-stuffing (RFC 5321), CRLF line endings, EHLO capability parsing
- **Type-safe errors**: Every failure mode has a typed error variant
- **Host-testable**: Protocol logic runs on any platform via `SmtpTransport` trait
- **Feature-gated**: ESP-IDF transport behind `esp-idf` feature flag

## Quick Start

```rust
use esp_idf_smtp::{SmtpConfig, Email, TlsMode, SmtpClient};

let config = SmtpConfig::new("smtp.gmail.com", 465)
    .tls_mode(TlsMode::ImplicitTls)
    .credentials("user@gmail.com", "app-password")
    .timeout_ms(10_000);

let email = Email::builder()
    .from("device@example.com")
    .to("alert@example.com")
    .subject("Nightwatch Alert")
    .body("Safety monitor triggered an alert.")
    .build()?;

SmtpClient::new(config).send(&email)?;
```

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `esp-idf` | Yes | ESP-IDF transport via `esp_tls`. Adds `esp-idf-svc` dependency. |

Without `esp-idf`, only protocol types and the `SmtpTransport` trait are available — useful for host testing or providing your own transport.

```toml
# Host-only (no ESP-IDF)
[dependencies]
esp-idf-smtp = { version = "0.1", default-features = false }
```

## TLS Modes

```rust
use esp_idf_smtp::{SmtpConfig, TlsMode};

// Gmail (implicit TLS on 465)
let config = SmtpConfig::new("smtp.gmail.com", 465);

// Corporate relay (STARTTLS on 587)
let config = SmtpConfig::new("mail.corp.com", 587)
    .tls_mode(TlsMode::StartTls);

// Local test server (no TLS)
let config = SmtpConfig::new("localhost", 25)
    .tls_mode(TlsMode::Plain);

// Self-signed server
let config = SmtpConfig::new("mail.local", 465)
    .skip_cert_verification();
```

## Multiple Recipients

```rust
let email = Email::builder()
    .from("device@example.com")
    .to("ops@example.com")
    .to("oncall@example.com")
    .cc("manager@example.com")
    .bcc("audit@example.com")
    .subject("Alert")
    .body("Details here.")
    .build()?;
```

BCC recipients receive the email but are not included in message headers.

## Host Testing

The protocol engine operates against the `SmtpTransport` trait. Implement it with a mock to test SMTP logic without hardware:

```rust
cargo test --no-default-features
```

## Minimum Supported Rust Version

1.75

## License

Licensed under the Apache License, Version 2.0 ([LICENSE](LICENSE) or <http://www.apache.org/licenses/LICENSE-2.0>).

Unless required by applicable law or agreed to in writing, software distributed under this license is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND.
