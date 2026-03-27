use crate::config::{SmtpConfig, TlsMode};
use crate::email::Email;
use crate::error::SmtpError;
use crate::response::SmtpResponse;
use crate::transport::SmtpTransport;
use base64::prelude::*;
use log::{debug, warn};

/// SMTP response buffer size.
const RESPONSE_BUF_SIZE: usize = 512;

/// EHLO capabilities parsed from server response.
#[derive(Debug, Default)]
pub struct EhloCapabilities {
    /// Supported AUTH methods (e.g., ["PLAIN", "LOGIN"]).
    pub auth_methods: Vec<String>,
    /// Whether STARTTLS is advertised.
    pub starttls: bool,
    /// Maximum message size (0 = not advertised).
    pub max_size: usize,
}

impl EhloCapabilities {
    /// Parse capabilities from a multiline EHLO response.
    pub fn parse(response: &SmtpResponse) -> Self {
        let mut caps = Self::default();

        for line in response.message.split('\n') {
            let line = line.trim();
            let upper = line.to_uppercase();

            if upper.starts_with("AUTH ") {
                caps.auth_methods = upper[5..]
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();
            } else if upper == "STARTTLS" {
                caps.starttls = true;
            } else if upper.starts_with("SIZE ") {
                caps.max_size = upper[5..].trim().parse().unwrap_or(0);
            }
        }

        caps
    }
}

/// Read an SMTP response from the transport.
fn read_response<T: SmtpTransport>(transport: &mut T) -> Result<SmtpResponse, SmtpError> {
    let mut buf = [0u8; RESPONSE_BUF_SIZE];
    let mut data = Vec::new();

    loop {
        let n = transport
            .read(&mut buf)
            .map_err(|e| SmtpError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        if n == 0 {
            return Err(SmtpError::Io(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "connection closed",
            )));
        }

        data.extend_from_slice(&buf[..n]);

        if let Some(resp) = SmtpResponse::parse(&data) {
            debug!("S: {}", resp);
            return Ok(resp);
        }
    }
}

/// Send an SMTP command and read the response.
fn send_command<T: SmtpTransport>(
    transport: &mut T,
    command: &str,
) -> Result<SmtpResponse, SmtpError> {
    let line = format!("{}\r\n", command);
    debug!("C: {}", command);
    transport
        .write_all(line.as_bytes())
        .map_err(|e| SmtpError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    read_response(transport)
}

/// Expect a specific response code, returning an error if mismatched.
fn expect_code(response: &SmtpResponse, expected: u16) -> Result<(), SmtpError> {
    if response.code != expected {
        Err(SmtpError::Protocol {
            expected,
            got: response.clone(),
        })
    } else {
        Ok(())
    }
}

/// Run the full SMTP protocol session.
///
/// This is the core protocol engine that operates against any `SmtpTransport`.
/// Public so integration tests can use it with mock transports.
pub fn send_email<T: SmtpTransport>(
    transport: &mut T,
    config: &SmtpConfig,
    email: &Email,
) -> Result<(), SmtpError> {
    // 1. Read server greeting (220)
    let greeting = read_response(transport)?;
    if greeting.code != 220 {
        return Err(SmtpError::Protocol {
            expected: 220,
            got: greeting,
        });
    }

    // 2. EHLO
    let ehlo_resp = send_command(transport, "EHLO localhost")?;
    expect_code(&ehlo_resp, 250)?;
    let mut caps = EhloCapabilities::parse(&ehlo_resp);

    // 3. STARTTLS (if configured)
    if config.tls_mode == TlsMode::StartTls {
        if !caps.starttls {
            return Err(SmtpError::Tls {
                message: "server does not advertise STARTTLS".into(),
            });
        }

        let starttls_resp = send_command(transport, "STARTTLS")?;
        expect_code(&starttls_resp, 220)?;

        transport
            .upgrade_tls(&config.host, &config.tls_verify)
            .map_err(|e| SmtpError::Tls {
                message: e.to_string(),
            })?;

        // Re-EHLO after TLS upgrade
        let ehlo2_resp = send_command(transport, "EHLO localhost")?;
        expect_code(&ehlo2_resp, 250)?;
        caps = EhloCapabilities::parse(&ehlo2_resp);
    }

    // 4. AUTH (if credentials provided)
    if let Some(ref creds) = config.credentials {
        authenticate(transport, creds, &caps)?;
    }

    // 5. MAIL FROM
    let mail_from = format!("MAIL FROM:<{}>", email.from.to_envelope());
    let mail_resp = send_command(transport, &mail_from)?;
    if !mail_resp.is_success() {
        return Err(SmtpError::RejectedSender {
            address: email.from.email.clone(),
            code: mail_resp.code,
            message: mail_resp.message,
        });
    }

    // 6. RCPT TO (for all recipients: to + cc + bcc)
    for recipient in email.all_recipients() {
        let rcpt_to = format!("RCPT TO:<{}>", recipient.to_envelope());
        let rcpt_resp = send_command(transport, &rcpt_to)?;
        if rcpt_resp.code != 250 && rcpt_resp.code != 251 {
            return Err(SmtpError::RejectedRecipient {
                address: recipient.email.clone(),
                code: rcpt_resp.code,
                message: rcpt_resp.message,
            });
        }
    }

    // 7. DATA
    let data_resp = send_command(transport, "DATA")?;
    if data_resp.code != 354 {
        return Err(SmtpError::DataRejected {
            code: data_resp.code,
            message: data_resp.message,
        });
    }

    // 8. Headers + body
    let headers = email.headers();
    transport
        .write_all(headers.as_bytes())
        .map_err(|e| SmtpError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    transport
        .write_all(b"\r\n")
        .map_err(|e| SmtpError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    let body = email.formatted_body();
    transport
        .write_all(body.as_bytes())
        .map_err(|e| SmtpError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    // End of message
    transport
        .write_all(b".\r\n")
        .map_err(|e| SmtpError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    let msg_resp = read_response(transport)?;
    if !msg_resp.is_success() {
        return Err(SmtpError::DataRejected {
            code: msg_resp.code,
            message: msg_resp.message,
        });
    }

    // 9. QUIT
    let _ = send_command(transport, "QUIT");

    Ok(())
}

/// Perform SMTP authentication.
fn authenticate<T: SmtpTransport>(
    transport: &mut T,
    creds: &crate::config::Credentials,
    caps: &EhloCapabilities,
) -> Result<(), SmtpError> {
    // Prefer AUTH PLAIN (single round-trip)
    if caps.auth_methods.contains(&"PLAIN".to_string()) {
        auth_plain(transport, creds)
    } else if caps.auth_methods.contains(&"LOGIN".to_string()) {
        auth_login(transport, creds)
    } else if caps.auth_methods.is_empty() {
        // Some servers don't advertise AUTH but accept it after STARTTLS
        // Try PLAIN as default
        warn!("no AUTH methods advertised, trying PLAIN");
        auth_plain(transport, creds)
    } else {
        Err(SmtpError::NoAuthMethod {
            available: caps.auth_methods.clone(),
        })
    }
}

/// AUTH PLAIN: single command with \0authzid\0authcid\0password base64-encoded.
fn auth_plain<T: SmtpTransport>(
    transport: &mut T,
    creds: &crate::config::Credentials,
) -> Result<(), SmtpError> {
    // Format: \0username\0password (authzid is empty)
    let mut token = Vec::new();
    token.push(0u8); // authzid (empty)
    token.extend_from_slice(creds.username.as_bytes());
    token.push(0u8);
    token.extend_from_slice(creds.password.as_bytes());

    let encoded = BASE64_STANDARD.encode(&token);
    let command = format!("AUTH PLAIN {}", encoded);

    let resp = send_command(transport, &command)?;
    if resp.code != 235 {
        return Err(SmtpError::Auth {
            code: resp.code,
            message: resp.message,
        });
    }

    Ok(())
}

/// AUTH LOGIN: two-step exchange with base64-encoded username and password.
fn auth_login<T: SmtpTransport>(
    transport: &mut T,
    creds: &crate::config::Credentials,
) -> Result<(), SmtpError> {
    let resp = send_command(transport, "AUTH LOGIN")?;
    if resp.code != 334 {
        return Err(SmtpError::Auth {
            code: resp.code,
            message: resp.message,
        });
    }

    // Send username
    let user_b64 = BASE64_STANDARD.encode(creds.username.as_bytes());
    let resp = send_command(transport, &user_b64)?;
    if resp.code != 334 {
        return Err(SmtpError::Auth {
            code: resp.code,
            message: resp.message,
        });
    }

    // Send password
    let pass_b64 = BASE64_STANDARD.encode(creds.password.as_bytes());
    let resp = send_command(transport, &pass_b64)?;
    if resp.code != 235 {
        return Err(SmtpError::Auth {
            code: resp.code,
            message: resp.message,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ehlo_capabilities_parse_auth() {
        let resp = SmtpResponse {
            code: 250,
            message: "smtp.example.com\nSIZE 35882577\nAUTH LOGIN PLAIN\nSTARTTLS".into(),
        };
        let caps = EhloCapabilities::parse(&resp);
        assert_eq!(caps.auth_methods, vec!["LOGIN", "PLAIN"]);
        assert!(caps.starttls);
        assert_eq!(caps.max_size, 35882577);
    }

    #[test]
    fn test_ehlo_capabilities_parse_no_auth() {
        let resp = SmtpResponse {
            code: 250,
            message: "smtp.example.com\nSIZE 1000".into(),
        };
        let caps = EhloCapabilities::parse(&resp);
        assert!(caps.auth_methods.is_empty());
        assert!(!caps.starttls);
    }
}
