//! Lightweight SMTP client for ESP-IDF devices.
//!
//! Provides a type-safe SMTP client with support for implicit TLS, STARTTLS,
//! and plaintext connections. Authentication via AUTH PLAIN or AUTH LOGIN.
//!
//! # Feature Flags
//!
//! - `esp-idf` (default): Enables the ESP-IDF transport using `esp_tls`.
//!   Without this feature, only the protocol types and trait are available.
//!
//! # Example
//!
//! ```no_run
//! use esp_idf_smtp::{SmtpConfig, Email, TlsMode};
//!
//! let config = SmtpConfig::new("smtp.example.com", 465)
//!     .tls_mode(TlsMode::ImplicitTls)
//!     .credentials("user", "pass");
//!
//! let email = Email::builder()
//!     .from("device@example.com")
//!     .to("alert@example.com")
//!     .subject("Alert")
//!     .body("Something happened.")
//!     .build()
//!     .unwrap();
//! ```

pub mod config;
mod email;
mod error;
pub mod protocol;
mod response;
mod transport;

#[cfg(feature = "esp-idf")]
mod esp_transport;

#[cfg(feature = "esp-idf")]
mod client;

pub use config::{Credentials, SmtpConfig, TlsMode, TlsVerify};
pub use email::{Address, Email, EmailBuilder};
pub use error::SmtpError;
pub use response::SmtpResponse;
pub use transport::SmtpTransport;

#[cfg(feature = "esp-idf")]
pub use client::SmtpClient;
