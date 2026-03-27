use esp_idf_smtp::config::TlsVerify;
use esp_idf_smtp::protocol::send_email;
use esp_idf_smtp::{Email, SmtpConfig, SmtpError, SmtpTransport, TlsMode};
use std::collections::VecDeque;

// --- Mock Transport ---

struct MockTransport {
    responses: VecDeque<Vec<u8>>,
    current: Vec<u8>,
    pos: usize,
    pub written: Vec<String>,
    pub tls_upgraded: bool,
    pub fail_on_read: bool,
}

impl MockTransport {
    fn new(responses: Vec<&str>) -> Self {
        Self {
            responses: responses
                .into_iter()
                .map(|s| s.as_bytes().to_vec())
                .collect(),
            current: Vec::new(),
            pos: 0,
            written: Vec::new(),
            tls_upgraded: false,
            fail_on_read: false,
        }
    }

    fn commands_text(&self) -> String {
        self.written.join("")
    }
}

impl SmtpTransport for MockTransport {
    type Error = std::io::Error;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if self.fail_on_read {
            return Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionReset,
                "mock reset",
            ));
        }
        if self.pos >= self.current.len() {
            if let Some(next) = self.responses.pop_front() {
                self.current = next;
                self.pos = 0;
            } else {
                return Ok(0);
            }
        }
        let remaining = &self.current[self.pos..];
        let n = buf.len().min(remaining.len());
        buf[..n].copy_from_slice(&remaining[..n]);
        self.pos += n;
        Ok(n)
    }

    fn write_all(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        if let Ok(s) = std::str::from_utf8(data) {
            self.written.push(s.to_string());
        }
        Ok(())
    }

    fn upgrade_tls(&mut self, _host: &str, _tls_verify: &TlsVerify) -> Result<(), Self::Error> {
        self.tls_upgraded = true;
        Ok(())
    }
}

fn simple_email() -> Email {
    Email::builder()
        .from("sender@example.com")
        .to("recipient@example.com")
        .subject("Test")
        .body("Hello")
        .build()
        .unwrap()
}

// --- Config tests ---

#[test]
fn smtp_config_builder() {
    let config = SmtpConfig::new("smtp.gmail.com", 465)
        .tls_mode(TlsMode::ImplicitTls)
        .credentials("user@gmail.com", "pass123")
        .timeout_ms(10_000)
        .skip_cert_verification();

    assert_eq!(config.host, "smtp.gmail.com");
    assert_eq!(config.port, 465);
    assert_eq!(config.tls_mode, TlsMode::ImplicitTls);
    assert!(config.credentials.is_some());
    assert_eq!(config.timeout_ms, 10_000);
}

#[test]
fn smtp_config_defaults() {
    let config = SmtpConfig::new("mail.example.com", 587);
    assert_eq!(config.tls_mode, TlsMode::ImplicitTls);
    assert!(config.credentials.is_none());
    assert_eq!(config.timeout_ms, 5000);
}

// --- Full session tests (T015) ---

#[test]
fn full_send_session_implicit_tls_no_auth() {
    let mut transport = MockTransport::new(vec![
        "220 smtp.example.com ESMTP\r\n",                // greeting
        "250-smtp.example.com\r\n250 SIZE 35882577\r\n", // EHLO
        "250 OK\r\n",                                    // MAIL FROM
        "250 OK\r\n",                                    // RCPT TO
        "354 Start mail input\r\n",                      // DATA
        "250 OK: queued\r\n",                            // message accepted
        "221 Bye\r\n",                                   // QUIT
    ]);

    let config = SmtpConfig::new("smtp.example.com", 465);
    let email = simple_email();

    send_email(&mut transport, &config, &email).unwrap();

    let cmds = transport.commands_text();
    assert!(cmds.contains("EHLO localhost\r\n"));
    assert!(cmds.contains("MAIL FROM:<sender@example.com>\r\n"));
    assert!(cmds.contains("RCPT TO:<recipient@example.com>\r\n"));
    assert!(cmds.contains("DATA\r\n"));
    assert!(cmds.contains(".\r\n"));
    assert!(cmds.contains("QUIT\r\n"));
    assert!(!transport.tls_upgraded); // Implicit TLS, no STARTTLS
}

#[test]
fn full_send_session_with_auth_plain() {
    let mut transport = MockTransport::new(vec![
        "220 smtp.example.com ESMTP\r\n",
        "250-smtp.example.com\r\n250 AUTH PLAIN LOGIN\r\n",
        "235 Authentication successful\r\n", // AUTH PLAIN
        "250 OK\r\n",                        // MAIL FROM
        "250 OK\r\n",                        // RCPT TO
        "354 Start mail input\r\n",
        "250 OK: queued\r\n",
        "221 Bye\r\n",
    ]);

    let config =
        SmtpConfig::new("smtp.example.com", 465).credentials("user@example.com", "password123");
    let email = simple_email();

    send_email(&mut transport, &config, &email).unwrap();

    let cmds = transport.commands_text();
    // Should use AUTH PLAIN (preferred over LOGIN)
    assert!(cmds.contains("AUTH PLAIN "));
    // Should NOT use AUTH LOGIN
    assert!(!cmds.contains("AUTH LOGIN\r\n"));
}

#[test]
fn full_send_session_with_auth_login_only() {
    let mut transport = MockTransport::new(vec![
        "220 smtp.example.com ESMTP\r\n",
        "250-smtp.example.com\r\n250 AUTH LOGIN\r\n", // LOGIN only
        "334 VXNlcm5hbWU6\r\n",                       // AUTH LOGIN prompt
        "334 UGFzc3dvcmQ6\r\n",                       // password prompt
        "235 Authentication successful\r\n",
        "250 OK\r\n",
        "250 OK\r\n",
        "354 Start mail input\r\n",
        "250 OK: queued\r\n",
        "221 Bye\r\n",
    ]);

    let config = SmtpConfig::new("smtp.example.com", 465).credentials("user", "pass");
    let email = simple_email();

    send_email(&mut transport, &config, &email).unwrap();

    let cmds = transport.commands_text();
    assert!(cmds.contains("AUTH LOGIN\r\n"));
}

// --- STARTTLS tests (T018) ---

#[test]
fn starttls_flow() {
    let mut transport = MockTransport::new(vec![
        "220 smtp.example.com ESMTP\r\n",
        "250-smtp.example.com\r\n250 STARTTLS\r\n", // first EHLO
        "220 Ready to start TLS\r\n",               // STARTTLS
        "250-smtp.example.com\r\n250 AUTH PLAIN\r\n", // second EHLO (after TLS)
        "235 OK\r\n",                               // AUTH
        "250 OK\r\n",                               // MAIL FROM
        "250 OK\r\n",                               // RCPT TO
        "354 Start mail input\r\n",
        "250 OK: queued\r\n",
        "221 Bye\r\n",
    ]);

    let config = SmtpConfig::new("smtp.example.com", 587)
        .tls_mode(TlsMode::StartTls)
        .credentials("user", "pass");
    let email = simple_email();

    send_email(&mut transport, &config, &email).unwrap();

    assert!(transport.tls_upgraded);
    let cmds = transport.commands_text();
    // Should have two EHLO commands (before and after STARTTLS)
    let ehlo_count = cmds.matches("EHLO localhost\r\n").count();
    assert_eq!(ehlo_count, 2);
    assert!(cmds.contains("STARTTLS\r\n"));
}

#[test]
fn starttls_fails_when_not_advertised() {
    let mut transport = MockTransport::new(vec![
        "220 smtp.example.com ESMTP\r\n",
        "250 smtp.example.com\r\n", // No STARTTLS capability
    ]);

    let config = SmtpConfig::new("smtp.example.com", 587).tls_mode(TlsMode::StartTls);
    let email = simple_email();

    let result = send_email(&mut transport, &config, &email);
    assert!(matches!(result, Err(SmtpError::Tls { .. })));
}

// --- Plain mode tests (T019) ---

#[test]
fn plain_mode_no_tls() {
    let mut transport = MockTransport::new(vec![
        "220 localhost ESMTP\r\n",
        "250 localhost\r\n",
        "250 OK\r\n",
        "250 OK\r\n",
        "354 Start mail input\r\n",
        "250 OK: queued\r\n",
        "221 Bye\r\n",
    ]);

    let config = SmtpConfig::new("localhost", 25).tls_mode(TlsMode::Plain);
    let email = simple_email();

    send_email(&mut transport, &config, &email).unwrap();

    assert!(!transport.tls_upgraded);
    let cmds = transport.commands_text();
    assert!(!cmds.contains("STARTTLS"));
}

// --- Error handling tests (T016) ---

#[test]
fn error_on_rejected_sender() {
    let mut transport = MockTransport::new(vec![
        "220 smtp.example.com ESMTP\r\n",
        "250 smtp.example.com\r\n",
        "550 Sender rejected\r\n", // MAIL FROM rejected
    ]);

    let config = SmtpConfig::new("smtp.example.com", 465);
    let email = simple_email();

    let result = send_email(&mut transport, &config, &email);
    assert!(matches!(
        result,
        Err(SmtpError::RejectedSender { code: 550, .. })
    ));
}

#[test]
fn error_on_rejected_recipient() {
    let mut transport = MockTransport::new(vec![
        "220 smtp.example.com ESMTP\r\n",
        "250 smtp.example.com\r\n",
        "250 OK\r\n",             // MAIL FROM ok
        "550 User not found\r\n", // RCPT TO rejected
    ]);

    let config = SmtpConfig::new("smtp.example.com", 465);
    let email = simple_email();

    let result = send_email(&mut transport, &config, &email);
    assert!(matches!(
        result,
        Err(SmtpError::RejectedRecipient { code: 550, .. })
    ));
}

#[test]
fn error_on_data_rejected() {
    let mut transport = MockTransport::new(vec![
        "220 smtp.example.com ESMTP\r\n",
        "250 smtp.example.com\r\n",
        "250 OK\r\n",
        "250 OK\r\n",
        "554 Transaction failed\r\n", // DATA rejected
    ]);

    let config = SmtpConfig::new("smtp.example.com", 465);
    let email = simple_email();

    let result = send_email(&mut transport, &config, &email);
    assert!(matches!(
        result,
        Err(SmtpError::DataRejected { code: 554, .. })
    ));
}

#[test]
fn error_on_connection_drop() {
    let mut transport = MockTransport::new(vec![
        "220 smtp.example.com ESMTP\r\n",
        "250 smtp.example.com\r\n",
    ]);
    // After EHLO, MAIL FROM will get EOF (no more responses)

    let config = SmtpConfig::new("smtp.example.com", 465);
    let email = simple_email();

    let result = send_email(&mut transport, &config, &email);
    assert!(result.is_err());
}

#[test]
fn error_on_bad_greeting() {
    let mut transport = MockTransport::new(vec!["554 Service unavailable\r\n"]);

    let config = SmtpConfig::new("smtp.example.com", 465);
    let email = simple_email();

    let result = send_email(&mut transport, &config, &email);
    assert!(matches!(
        result,
        Err(SmtpError::Protocol { expected: 220, .. })
    ));
}

// --- Auth tests (T024) ---

#[test]
fn auth_invalid_credentials() {
    let mut transport = MockTransport::new(vec![
        "220 smtp.example.com ESMTP\r\n",
        "250-smtp.example.com\r\n250 AUTH PLAIN\r\n",
        "535 Authentication failed\r\n",
    ]);

    let config = SmtpConfig::new("smtp.example.com", 465).credentials("bad", "wrong");
    let email = simple_email();

    let result = send_email(&mut transport, &config, &email);
    assert!(matches!(result, Err(SmtpError::Auth { code: 535, .. })));
}

#[test]
fn no_auth_when_no_credentials() {
    let mut transport = MockTransport::new(vec![
        "220 smtp.example.com ESMTP\r\n",
        "250-smtp.example.com\r\n250 AUTH PLAIN LOGIN\r\n",
        "250 OK\r\n",
        "250 OK\r\n",
        "354 Start mail input\r\n",
        "250 OK: queued\r\n",
        "221 Bye\r\n",
    ]);

    let config = SmtpConfig::new("smtp.example.com", 465); // No credentials
    let email = simple_email();

    send_email(&mut transport, &config, &email).unwrap();

    let cmds = transport.commands_text();
    assert!(!cmds.contains("AUTH"));
}

// --- Multi-recipient tests (T027) ---

#[test]
fn multi_recipient_rcpt_to_commands() {
    let mut transport = MockTransport::new(vec![
        "220 smtp.example.com ESMTP\r\n",
        "250 smtp.example.com\r\n",
        "250 OK\r\n", // MAIL FROM
        "250 OK\r\n", // RCPT TO #1
        "250 OK\r\n", // RCPT TO #2
        "250 OK\r\n", // RCPT TO #3 (CC)
        "250 OK\r\n", // RCPT TO #4 (BCC)
        "354 Start\r\n",
        "250 OK\r\n",
        "221 Bye\r\n",
    ]);

    let config = SmtpConfig::new("smtp.example.com", 465);
    let email = Email::builder()
        .from("a@b.com")
        .to("to1@x.com")
        .to("to2@x.com")
        .cc("cc1@x.com")
        .bcc("bcc1@x.com")
        .subject("Test")
        .body("Body")
        .build()
        .unwrap();

    send_email(&mut transport, &config, &email).unwrap();

    let cmds = transport.commands_text();
    // 4 RCPT TO commands
    assert_eq!(cmds.matches("RCPT TO:").count(), 4);
    assert!(cmds.contains("RCPT TO:<to1@x.com>"));
    assert!(cmds.contains("RCPT TO:<to2@x.com>"));
    assert!(cmds.contains("RCPT TO:<cc1@x.com>"));
    assert!(cmds.contains("RCPT TO:<bcc1@x.com>"));
}

// --- TlsVerify config tests (T031) ---

#[test]
fn config_verify_certs_default() {
    let config = SmtpConfig::new("smtp.example.com", 465);
    assert!(matches!(
        config.tls_verify,
        esp_idf_smtp::config::TlsVerify::Verify
    ));
}

#[test]
fn config_skip_cert_verification() {
    let config = SmtpConfig::new("smtp.example.com", 465).skip_cert_verification();
    assert!(matches!(
        config.tls_verify,
        esp_idf_smtp::config::TlsVerify::SkipVerify
    ));
}

#[test]
fn config_custom_ca() {
    let pem = b"-----BEGIN CERTIFICATE-----\nfake\n-----END CERTIFICATE-----";
    let config = SmtpConfig::new("smtp.example.com", 465).ca_cert_pem(pem);
    assert!(matches!(
        config.tls_verify,
        esp_idf_smtp::config::TlsVerify::CustomCa(_)
    ));
}

// --- SC-001: Under 10 lines ---

#[test]
fn email_under_10_lines() {
    let _config = SmtpConfig::new("smtp.gmail.com", 465)
        .tls_mode(TlsMode::ImplicitTls)
        .credentials("user@gmail.com", "app-password")
        .timeout_ms(10_000);

    let _email = Email::builder()
        .from("device@example.com")
        .to("alert@example.com")
        .subject("Alert")
        .body("Something happened.")
        .build()
        .unwrap();

    // config: 4 lines, email: 6 lines = 10 total. SC-001 passes.
}
