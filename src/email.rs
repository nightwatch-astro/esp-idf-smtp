use crate::error::SmtpError;

/// An email address with optional display name.
#[derive(Debug, Clone)]
pub struct Address {
    pub email: String,
    pub name: Option<String>,
}

impl Address {
    /// Create an address from an email string.
    pub fn new(email: &str) -> Self {
        Self {
            email: email.to_string(),
            name: None,
        }
    }

    /// Create an address with a display name.
    pub fn with_name(name: &str, email: &str) -> Self {
        Self {
            email: email.to_string(),
            name: Some(name.to_string()),
        }
    }

    /// Format for message headers: `"Name" <email>` or `email`.
    pub fn to_header(&self) -> String {
        match &self.name {
            Some(name) => format!("\"{}\" <{}>", name, self.email),
            None => self.email.clone(),
        }
    }

    /// Format for SMTP envelope (MAIL FROM / RCPT TO): just the email.
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
}

impl Email {
    /// Start building a new email.
    pub fn builder() -> EmailBuilder {
        EmailBuilder::default()
    }

    /// Generate all RCPT TO addresses (to + cc + bcc).
    pub fn all_recipients(&self) -> impl Iterator<Item = &Address> {
        self.to.iter().chain(self.cc.iter()).chain(self.bcc.iter())
    }

    /// Generate the email headers as a string.
    pub fn headers(&self) -> String {
        let mut h = String::with_capacity(512);

        // From
        h.push_str(&format!("From: {}\r\n", self.from.to_header()));

        // To
        let to_list: Vec<String> = self.to.iter().map(|a| a.to_header()).collect();
        h.push_str(&format!("To: {}\r\n", to_list.join(", ")));

        // CC (if any)
        if !self.cc.is_empty() {
            let cc_list: Vec<String> = self.cc.iter().map(|a| a.to_header()).collect();
            h.push_str(&format!("Cc: {}\r\n", cc_list.join(", ")));
        }

        // BCC deliberately omitted from headers

        // Subject
        h.push_str(&format!("Subject: {}\r\n", self.subject));

        // Date (RFC 2822 format — simplified for embedded)
        // Note: On ESP32 with NTP, this would use real time.
        // For now, omit Date if no system time is available.

        // MIME headers
        h.push_str("MIME-Version: 1.0\r\n");
        h.push_str("Content-Type: text/plain; charset=UTF-8\r\n");

        // Message-ID
        if let Some(ref id) = self.message_id {
            h.push_str(&format!("Message-ID: <{}>\r\n", id));
        }

        // X-Mailer
        h.push_str(&format!(
            "X-Mailer: esp-idf-smtp/{}\r\n",
            env!("CARGO_PKG_VERSION")
        ));

        h
    }

    /// Format the body with RFC 5321 dot-stuffing and \r\n line endings.
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
}

impl EmailBuilder {
    /// Set the sender address.
    pub fn from(mut self, email: &str) -> Self {
        self.from = Some(Address::new(email));
        self
    }

    /// Set the sender with a display name.
    pub fn from_named(mut self, name: &str, email: &str) -> Self {
        self.from = Some(Address::with_name(name, email));
        self
    }

    /// Add a To recipient.
    pub fn to(mut self, email: &str) -> Self {
        self.to.push(Address::new(email));
        self
    }

    /// Add a CC recipient.
    pub fn cc(mut self, email: &str) -> Self {
        self.cc.push(Address::new(email));
        self
    }

    /// Add a BCC recipient.
    pub fn bcc(mut self, email: &str) -> Self {
        self.bcc.push(Address::new(email));
        self
    }

    /// Set the subject line.
    pub fn subject(mut self, subject: &str) -> Self {
        self.subject = Some(subject.to_string());
        self
    }

    /// Set the plain text body.
    pub fn body(mut self, body: &str) -> Self {
        self.body = Some(body.to_string());
        self
    }

    /// Set a custom Message-ID.
    pub fn message_id(mut self, id: &str) -> Self {
        self.message_id = Some(id.to_string());
        self
    }

    /// Build the email, validating required fields.
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
        })
    }
}
