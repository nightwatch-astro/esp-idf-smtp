use crate::config::SmtpConfig;
use crate::email::Email;
use crate::error::SmtpError;

/// SMTP client for sending emails.
///
/// Each `send()` call opens a new connection and closes it after QUIT.
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
    pub fn send(&self, _email: &Email) -> Result<(), SmtpError> {
        // TODO: Wire to EspTransport in Phase 9 (T033)
        let _ = &self.config;
        unimplemented!("ESP SmtpClient not yet wired — use protocol::send_email with mock transport for host testing")
    }
}
