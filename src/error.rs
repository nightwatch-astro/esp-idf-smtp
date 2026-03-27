use crate::response::SmtpResponse;

/// Errors that can occur during SMTP operations.
#[derive(Debug, thiserror::Error)]
pub enum SmtpError {
    /// TCP connection failed.
    #[error("connection to {host}:{port} failed: {source}")]
    Connection {
        host: String,
        port: u16,
        source: std::io::Error,
    },

    /// TLS handshake or upgrade failed.
    #[error("TLS error: {message}")]
    Tls { message: String },

    /// Authentication rejected by server.
    #[error("authentication failed (code {code}): {message}")]
    Auth { code: u16, message: String },

    /// MAIL FROM rejected by server.
    #[error("sender {address} rejected (code {code}): {message}")]
    RejectedSender {
        address: String,
        code: u16,
        message: String,
    },

    /// RCPT TO rejected by server.
    #[error("recipient {address} rejected (code {code}): {message}")]
    RejectedRecipient {
        address: String,
        code: u16,
        message: String,
    },

    /// DATA or message body rejected by server.
    #[error("data rejected (code {code}): {message}")]
    DataRejected { code: u16, message: String },

    /// SMTP command timed out.
    #[error("timeout waiting for response to {stage}")]
    Timeout { stage: String },

    /// Unexpected SMTP response code.
    #[error("protocol error: expected {expected}, got {}", got.code)]
    Protocol { expected: u16, got: SmtpResponse },

    /// Email builder validation failed.
    #[error("invalid email: {message}")]
    InvalidEmail { message: String },

    /// No compatible authentication method available.
    #[error("no compatible auth method (server supports: {available:?})")]
    NoAuthMethod { available: Vec<String> },

    /// Underlying I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
