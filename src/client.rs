use crate::config::SmtpConfig;
use crate::email::Email;
use crate::error::SmtpError;
use crate::esp_transport::EspTransport;
use crate::protocol;

/// SMTP client for sending emails.
///
/// Each `send()` call opens a new connection and closes it after QUIT.
/// Uses `EspTransport` (esp_tls) for the underlying connection.
pub struct SmtpClient {
    config: SmtpConfig,
}

impl SmtpClient {
    /// Create a new SMTP client with the given configuration.
    pub fn new(config: SmtpConfig) -> Self {
        Self { config }
    }

    /// Send an email.
    ///
    /// Opens a connection, runs the SMTP protocol, and closes the connection.
    /// The connection is cleaned up (QUIT sent) on both success and error paths.
    pub fn send(&self, email: &Email) -> Result<(), SmtpError> {
        let mut transport = EspTransport::connect(&self.config).map_err(|e| {
            SmtpError::Connection {
                host: self.config.host.clone(),
                port: self.config.port,
                source: e,
            }
        })?;

        protocol::send_email(&mut transport, &self.config, email)
    }
}
