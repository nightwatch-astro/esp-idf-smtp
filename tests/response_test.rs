use esp_idf_smtp::SmtpResponse;

#[test]
fn parse_single_line_response() {
    let data = b"220 smtp.example.com ESMTP ready\r\n";
    let resp = SmtpResponse::parse(data).unwrap();
    assert_eq!(resp.code, 220);
    assert_eq!(resp.message, "smtp.example.com ESMTP ready");
}

#[test]
fn parse_multiline_response() {
    let data =
        b"250-smtp.example.com\r\n250-SIZE 35882577\r\n250-AUTH LOGIN PLAIN\r\n250 STARTTLS\r\n";
    let resp = SmtpResponse::parse(data).unwrap();
    assert_eq!(resp.code, 250);
    assert!(resp.message.contains("smtp.example.com"));
    assert!(resp.message.contains("AUTH LOGIN PLAIN"));
    assert!(resp.message.contains("STARTTLS"));
}

#[test]
fn parse_error_response() {
    let data = b"550 User not found\r\n";
    let resp = SmtpResponse::parse(data).unwrap();
    assert_eq!(resp.code, 550);
    assert_eq!(resp.message, "User not found");
    assert!(!resp.is_success());
}

#[test]
fn parse_intermediate_response() {
    let data = b"334 VXNlcm5hbWU6\r\n";
    let resp = SmtpResponse::parse(data).unwrap();
    assert_eq!(resp.code, 334);
    assert!(resp.is_intermediate());
}

#[test]
fn parse_success_codes() {
    let data = b"250 OK\r\n";
    let resp = SmtpResponse::parse(data).unwrap();
    assert!(resp.is_success());

    let data = b"235 Authentication successful\r\n";
    let resp = SmtpResponse::parse(data).unwrap();
    assert!(resp.is_success());
}

#[test]
fn parse_empty_returns_none() {
    assert!(SmtpResponse::parse(b"").is_none());
}

#[test]
fn parse_incomplete_returns_none() {
    // No final line yet (all continuation)
    assert!(SmtpResponse::parse(b"250-first\r\n250-second\r\n").is_none());
}

#[test]
fn parse_malformed_returns_none() {
    assert!(SmtpResponse::parse(b"abc bad response\r\n").is_none());
}

#[test]
fn parse_three_digit_only() {
    let data = b"250\r\n";
    let resp = SmtpResponse::parse(data).unwrap();
    assert_eq!(resp.code, 250);
}
