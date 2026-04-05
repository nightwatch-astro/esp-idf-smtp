use std::fmt::Write as _;

use crate::error::SmtpError;

/// An email address with optional display name.
#[derive(Debug, Clone)]
pub struct Address {
    pub email: String,
    pub name: Option<String>,
}

impl Address {
    /// Create an address from an email string.
    #[must_use]
    pub fn new(email: &str) -> Self {
        Self {
            email: email.to_string(),
            name: None,
        }
    }

    /// Create an address with a display name.
    #[must_use]
    pub fn with_name(name: &str, email: &str) -> Self {
        Self {
            email: email.to_string(),
            name: Some(name.to_string()),
        }
    }

    /// Format for message headers: `"Name" <email>` or `email`.
    #[must_use]
    pub fn to_header(&self) -> String {
        self.name.as_ref().map_or_else(
            || self.email.clone(),
            |name| format!("\"{}\" <{}>", name, self.email),
        )
    }

    /// Format for SMTP envelope (MAIL FROM / RCPT TO): just the email.
    #[must_use]
    pub fn to_envelope(&self) -> &str {
        &self.email
    }
}

/// A composed email message.
#[derive(Debug, Clone)]
pub struct Email {
    pub from: Address,
    pub to: Vec<Address>,
    pub cc: Vec<Address>,
    pub bcc: Vec<Address>,
    pub subject: String,
    pub body: String,
    pub message_id: Option<String>,
    /// Optional Date header value (RFC 2822 format).
    /// If None, no Date header is emitted.
    pub date: Option<String>,
}

impl Email {
    /// Start building a new email.
    #[must_use]
    pub fn builder() -> EmailBuilder {
        EmailBuilder::default()
    }

    /// Generate all RCPT TO addresses (to + cc + bcc).
    pub fn all_recipients(&self) -> impl Iterator<Item = &Address> {
        self.to.iter().chain(self.cc.iter()).chain(self.bcc.iter())
    }

    /// Generate the email headers as a string.
    #[must_use]
    pub fn headers(&self) -> String {
        let mut h = String::with_capacity(512);

        // From
        let _ = write!(h, "From: {}\r\n", self.from.to_header());

        // To
        let to_list: Vec<String> = self.to.iter().map(Address::to_header).collect();
        let _ = write!(h, "To: {}\r\n", to_list.join(", "));

        // CC (if any)
        if !self.cc.is_empty() {
            let cc_list: Vec<String> = self.cc.iter().map(Address::to_header).collect();
            let _ = write!(h, "Cc: {}\r\n", cc_list.join(", "));
        }

        // BCC deliberately omitted from headers

        // Subject
        let _ = write!(h, "Subject: {}\r\n", self.subject);

        // Date (caller-provided, RFC 2822 format)
        if let Some(ref date) = self.date {
            let _ = write!(h, "Date: {date}\r\n");
        }

        // MIME headers
        h.push_str("MIME-Version: 1.0\r\n");
        h.push_str("Content-Type: text/plain; charset=UTF-8\r\n");

        // Message-ID
        if let Some(ref id) = self.message_id {
            let _ = write!(h, "Message-ID: <{id}>\r\n");
        }

        // X-Mailer
        let _ = write!(
            h,
            "X-Mailer: esp-idf-smtp/{}\r\n",
            env!("CARGO_PKG_VERSION")
        );

        h
    }

    /// Format the body with RFC 5321 dot-stuffing and \r\n line endings.
    #[must_use]
    pub fn formatted_body(&self) -> String {
        let mut out = String::with_capacity(self.body.len() + 64);

        for line in self.body.split('\n') {
            // Strip trailing \r if present (normalize from \r\n input)
            let line = line.strip_suffix('\r').unwrap_or(line);

            // Dot-stuffing: lines starting with '.' get an extra '.'
            if line.starts_with('.') {
                out.push('.');
            }
            out.push_str(line);
            out.push_str("\r\n");
        }

        out
    }
}

/// Builder for composing emails.
#[derive(Debug, Default)]
pub struct EmailBuilder {
    from: Option<Address>,
    to: Vec<Address>,
    cc: Vec<Address>,
    bcc: Vec<Address>,
    subject: Option<String>,
    body: Option<String>,
    message_id: Option<String>,
    date: Option<String>,
}

impl EmailBuilder {
    /// Set the sender address.
    #[must_use]
    pub fn from(mut self, email: &str) -> Self {
        self.from = Some(Address::new(email));
        self
    }

    /// Set the sender with a display name.
    #[must_use]
    pub fn from_named(mut self, name: &str, email: &str) -> Self {
        self.from = Some(Address::with_name(name, email));
        self
    }

    /// Add a To recipient.
    #[must_use]
    pub fn to(mut self, email: &str) -> Self {
        self.to.push(Address::new(email));
        self
    }

    /// Add a CC recipient.
    #[must_use]
    pub fn cc(mut self, email: &str) -> Self {
        self.cc.push(Address::new(email));
        self
    }

    /// Add a BCC recipient.
    #[must_use]
    pub fn bcc(mut self, email: &str) -> Self {
        self.bcc.push(Address::new(email));
        self
    }

    /// Set the subject line.
    #[must_use]
    pub fn subject(mut self, subject: &str) -> Self {
        self.subject = Some(subject.to_string());
        self
    }

    /// Set the plain text body.
    #[must_use]
    pub fn body(mut self, body: &str) -> Self {
        self.body = Some(body.to_string());
        self
    }

    /// Set a custom Message-ID.
    #[must_use]
    pub fn message_id(mut self, id: &str) -> Self {
        self.message_id = Some(id.to_string());
        self
    }

    /// Set the Date header value (RFC 2822 format, e.g. "Fri, 28 Mar 2026 12:00:00 +0000").
    /// If not set, no Date header is emitted.
    #[must_use]
    pub fn date(mut self, date: &str) -> Self {
        self.date = Some(date.to_string());
        self
    }

    /// Build the email, validating required fields.
    ///
    /// # Errors
    ///
    /// Returns [`SmtpError::InvalidEmail`] if any required field (`from`,
    /// `to`, `subject`, `body`) is missing.
    pub fn build(self) -> Result<Email, SmtpError> {
        let from = self.from.ok_or_else(|| SmtpError::InvalidEmail {
            message: "from address is required".into(),
        })?;

        if self.to.is_empty() {
            return Err(SmtpError::InvalidEmail {
                message: "at least one To recipient is required".into(),
            });
        }

        let subject = self.subject.ok_or_else(|| SmtpError::InvalidEmail {
            message: "subject is required".into(),
        })?;

        let body = self.body.ok_or_else(|| SmtpError::InvalidEmail {
            message: "body is required".into(),
        })?;

        Ok(Email {
            from,
            to: self.to,
            cc: self.cc,
            bcc: self.bcc,
            subject,
            body,
            message_id: self.message_id,
            date: self.date,
        })
    }
}
