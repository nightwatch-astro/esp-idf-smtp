/// A parsed SMTP server response.
#[derive(Debug, Clone)]
pub struct SmtpResponse {
    /// 3-digit SMTP status code.
    pub code: u16,
    /// Response text (may be multiline, joined with newlines).
    pub message: String,
}

impl std::fmt::Display for SmtpResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.code, self.message)
    }
}

impl SmtpResponse {
    /// Parse an SMTP response from raw bytes.
    ///
    /// SMTP responses have the format:
    /// - `250-First line\r\n` (continuation)
    /// - `250 Last line\r\n` (final)
    ///
    /// Returns the parsed response once a final line is seen.
    pub fn parse(data: &[u8]) -> Option<Self> {
        let text = std::str::from_utf8(data).ok()?;
        let mut code: Option<u16> = None;
        let mut messages = Vec::new();
        let mut found_final = false;

        for line in text.lines() {
            if line.len() < 3 {
                continue;
            }

            let line_code = line[..3].parse::<u16>().ok()?;

            if code.is_none() {
                code = Some(line_code);
            } else if code != Some(line_code) {
                return None; // Inconsistent codes
            }

            if line.len() >= 4 {
                let separator = line.as_bytes()[3];
                let msg = if line.len() > 4 { &line[4..] } else { "" };
                messages.push(msg.to_string());

                if separator == b' ' {
                    found_final = true;
                    break;
                }
                // separator == b'-' means continuation
            } else {
                // Exactly 3 chars = final line with no message
                found_final = true;
                break;
            }
        }

        if found_final {
            Some(SmtpResponse {
                code: code?,
                message: messages.join("\n"),
            })
        } else {
            None
        }
    }

    /// Check if this response indicates success (2xx).
    pub fn is_success(&self) -> bool {
        self.code >= 200 && self.code < 300
    }

    /// Check if this response is a positive intermediate reply (3xx).
    pub fn is_intermediate(&self) -> bool {
        self.code >= 300 && self.code < 400
    }
}
