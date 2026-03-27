use crate::config::TlsVerify;
use crate::transport::SmtpTransport;
use std::collections::VecDeque;

/// Mock transport for host testing.
///
/// Records all commands written and returns scripted responses.
#[derive(Debug)]
pub struct MockTransport {
    /// Scripted responses to return on read(), in order.
    responses: VecDeque<Vec<u8>>,
    /// Current response being read from.
    current: Vec<u8>,
    /// Position in current response.
    pos: usize,
    /// All data written (commands sent).
    pub written: Vec<String>,
    /// Whether upgrade_tls was called.
    pub tls_upgraded: bool,
    /// TlsVerify passed to upgrade_tls.
    pub tls_verify_used: Option<TlsVerify>,
    /// If set, the next read will return this error.
    pub fail_on_read: bool,
}

impl MockTransport {
    /// Create a new mock transport with scripted responses.
    pub fn new(responses: Vec<&str>) -> Self {
        Self {
            responses: responses.into_iter().map(|s| s.as_bytes().to_vec()).collect(),
            current: Vec::new(),
            pos: 0,
            written: Vec::new(),
            tls_upgraded: false,
            tls_verify_used: None,
            fail_on_read: false,
        }
    }

    /// Get all commands that were written, split by \r\n.
    pub fn commands(&self) -> Vec<&str> {
        self.written.iter().map(|s| s.as_str()).collect()
    }
}

impl SmtpTransport for MockTransport {
    type Error = std::io::Error;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if self.fail_on_read {
            return Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionReset,
                "mock connection reset",
            ));
        }

        // If we've exhausted current response, get next one
        if self.pos >= self.current.len() {
            if let Some(next) = self.responses.pop_front() {
                self.current = next;
                self.pos = 0;
            } else {
                return Ok(0); // No more data
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

    fn upgrade_tls(&mut self, _host: &str, tls_verify: &TlsVerify) -> Result<(), Self::Error> {
        self.tls_upgraded = true;
        self.tls_verify_used = Some(tls_verify.clone());
        Ok(())
    }
}
