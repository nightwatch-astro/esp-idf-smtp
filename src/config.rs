/// TLS connection mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsMode {
    /// TLS from connection start (typically port 465).
    ImplicitTls,
    /// Upgrade plaintext to TLS after EHLO (typically port 587).
    StartTls,
    /// No encryption (for trusted local relays, typically port 25).
    Plain,
}

/// TLS certificate verification mode.
#[derive(Debug, Clone, Default)]
pub enum TlsVerify {
    /// Verify server certificate against system CA bundle (default).
    #[default]
    Verify,
    /// Accept any certificate (for self-signed servers).
    SkipVerify,
    /// Verify against a custom CA certificate in PEM format.
    CustomCa(Vec<u8>),
}

/// SMTP authentication credentials.
#[derive(Debug, Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

/// SMTP connection configuration.
#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub tls_mode: TlsMode,
    pub credentials: Option<Credentials>,
    pub timeout_ms: u32,
    pub tls_verify: TlsVerify,
}

impl SmtpConfig {
    /// Create a new SMTP configuration.
    ///
    /// Defaults: implicit TLS, no credentials, 5s timeout, verify certs.
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
            tls_mode: TlsMode::ImplicitTls,
            credentials: None,
            timeout_ms: 5000,
            tls_verify: TlsVerify::default(),
        }
    }

    /// Set the TLS mode.
    pub fn tls_mode(mut self, mode: TlsMode) -> Self {
        self.tls_mode = mode;
        self
    }

    /// Set authentication credentials.
    pub fn credentials(mut self, username: &str, password: &str) -> Self {
        self.credentials = Some(Credentials {
            username: username.to_string(),
            password: password.to_string(),
        });
        self
    }

    /// Set per-command timeout in milliseconds.
    pub fn timeout_ms(mut self, ms: u32) -> Self {
        self.timeout_ms = ms;
        self
    }

    /// Enable certificate verification (default).
    pub fn verify_certs(mut self) -> Self {
        self.tls_verify = TlsVerify::Verify;
        self
    }

    /// Skip certificate verification.
    pub fn skip_cert_verification(mut self) -> Self {
        self.tls_verify = TlsVerify::SkipVerify;
        self
    }

    /// Use a custom CA certificate in PEM format.
    pub fn ca_cert_pem(mut self, pem: &[u8]) -> Self {
        self.tls_verify = TlsVerify::CustomCa(pem.to_vec());
        self
    }
}
