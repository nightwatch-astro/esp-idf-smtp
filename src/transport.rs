use crate::config::TlsVerify;

/// Abstraction over I/O for SMTP protocol operations.
///
/// Implement this trait to provide custom transport (e.g., for testing).
/// The ESP-IDF implementation is provided behind the `esp-idf` feature flag.
pub trait SmtpTransport {
    /// The error type for transport operations.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Read bytes from the connection into `buf`.
    /// Returns the number of bytes read.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` on I/O or connection failure.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error>;

    /// Write all bytes in `data` to the connection.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` on I/O or connection failure.
    fn write_all(&mut self, data: &[u8]) -> Result<(), Self::Error>;

    /// Upgrade the connection to TLS (for STARTTLS).
    ///
    /// `host` is the server hostname for SNI.
    /// `tls_verify` controls certificate verification.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` if the TLS handshake fails.
    fn upgrade_tls(&mut self, host: &str, tls_verify: &TlsVerify) -> Result<(), Self::Error>;
}
