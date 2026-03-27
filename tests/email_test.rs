use esp_idf_smtp::{Address, Email, SmtpError};

#[test]
fn build_valid_email() {
    let email = Email::builder()
        .from("sender@example.com")
        .to("recipient@example.com")
        .subject("Test Subject")
        .body("Hello, world!")
        .build()
        .unwrap();

    assert_eq!(email.from.email, "sender@example.com");
    assert_eq!(email.to.len(), 1);
    assert_eq!(email.subject, "Test Subject");
    assert_eq!(email.body, "Hello, world!");
}

#[test]
fn build_missing_from_fails() {
    let result = Email::builder()
        .to("recipient@example.com")
        .subject("Test")
        .body("Body")
        .build();

    assert!(matches!(result, Err(SmtpError::InvalidEmail { .. })));
}

#[test]
fn build_missing_to_fails() {
    let result = Email::builder()
        .from("sender@example.com")
        .subject("Test")
        .body("Body")
        .build();

    assert!(matches!(result, Err(SmtpError::InvalidEmail { .. })));
}

#[test]
fn build_missing_subject_fails() {
    let result = Email::builder()
        .from("sender@example.com")
        .to("recipient@example.com")
        .body("Body")
        .build();

    assert!(matches!(result, Err(SmtpError::InvalidEmail { .. })));
}

#[test]
fn build_missing_body_fails() {
    let result = Email::builder()
        .from("sender@example.com")
        .to("recipient@example.com")
        .subject("Subject")
        .build();

    assert!(matches!(result, Err(SmtpError::InvalidEmail { .. })));
}

#[test]
fn dot_stuffing_applied() {
    let email = Email::builder()
        .from("a@b.com")
        .to("c@d.com")
        .subject("Test")
        .body(".leading dot\nnormal line\n..double dot")
        .build()
        .unwrap();

    let body = email.formatted_body();
    assert!(body.contains("..leading dot\r\n"));
    assert!(body.contains("normal line\r\n"));
    assert!(body.contains("...double dot\r\n"));
}

#[test]
fn line_ending_normalization() {
    let email = Email::builder()
        .from("a@b.com")
        .to("c@d.com")
        .subject("Test")
        .body("line1\nline2\r\nline3")
        .build()
        .unwrap();

    let body = email.formatted_body();
    // All lines should end with \r\n, no bare \n
    for line in body.split("\r\n") {
        assert!(!line.contains('\n'), "bare \\n found in: {:?}", line);
    }
}

#[test]
fn headers_contain_required_fields() {
    let email = Email::builder()
        .from("sender@example.com")
        .to("recipient@example.com")
        .subject("Test Subject")
        .body("Body")
        .build()
        .unwrap();

    let headers = email.headers();
    assert!(headers.contains("From: sender@example.com\r\n"));
    assert!(headers.contains("To: recipient@example.com\r\n"));
    assert!(headers.contains("Subject: Test Subject\r\n"));
    assert!(headers.contains("MIME-Version: 1.0\r\n"));
    assert!(headers.contains("Content-Type: text/plain; charset=UTF-8\r\n"));
    assert!(headers.contains("X-Mailer: esp-idf-smtp/"));
}

#[test]
fn headers_omit_bcc() {
    let email = Email::builder()
        .from("a@b.com")
        .to("c@d.com")
        .bcc("secret@hidden.com")
        .subject("Test")
        .body("Body")
        .build()
        .unwrap();

    let headers = email.headers();
    assert!(!headers.contains("secret@hidden.com"));
    assert!(!headers.contains("Bcc"));
}

#[test]
fn headers_include_cc() {
    let email = Email::builder()
        .from("a@b.com")
        .to("c@d.com")
        .cc("copy@example.com")
        .subject("Test")
        .body("Body")
        .build()
        .unwrap();

    let headers = email.headers();
    assert!(headers.contains("Cc: copy@example.com\r\n"));
}

#[test]
fn all_recipients_includes_to_cc_bcc() {
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

    let recipients: Vec<&str> = email.all_recipients().map(|a| a.email.as_str()).collect();
    assert_eq!(recipients.len(), 4);
    assert!(recipients.contains(&"to1@x.com"));
    assert!(recipients.contains(&"to2@x.com"));
    assert!(recipients.contains(&"cc1@x.com"));
    assert!(recipients.contains(&"bcc1@x.com"));
}

#[test]
fn address_header_format_plain() {
    let addr = Address::new("test@example.com");
    assert_eq!(addr.to_header(), "test@example.com");
    assert_eq!(addr.to_envelope(), "test@example.com");
}

#[test]
fn address_header_format_with_name() {
    let addr = Address::with_name("John Doe", "john@example.com");
    assert_eq!(addr.to_header(), "\"John Doe\" <john@example.com>");
    assert_eq!(addr.to_envelope(), "john@example.com");
}

#[test]
fn custom_message_id() {
    let email = Email::builder()
        .from("a@b.com")
        .to("c@d.com")
        .subject("Test")
        .body("Body")
        .message_id("unique-123@example.com")
        .build()
        .unwrap();

    let headers = email.headers();
    assert!(headers.contains("Message-ID: <unique-123@example.com>\r\n"));
}
