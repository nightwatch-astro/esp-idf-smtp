// ESP-IDF transport implementation.
// TODO: Implement in Phase 9 (T032-T034).

use crate::config::TlsVerify;
use crate::transport::SmtpTransport;

/// ESP-IDF transport using `esp_tls` for TLS and `std::net::TcpStream` for plaintext.
pub struct EspTransport {
    // Will hold either TcpStream or esp_tls handle
    _placeholder: (),
}

impl SmtpTransport for EspTransport {
    type Error = std::io::Error;

    fn read(&mut self, _buf: &mut [u8]) -> Result<usize, Self::Error> {
        unimplemented!("ESP transport not yet implemented — use mock transport for host testing")
    }

    fn write_all(&mut self, _data: &[u8]) -> Result<(), Self::Error> {
        unimplemented!("ESP transport not yet implemented — use mock transport for host testing")
    }

    fn upgrade_tls(&mut self, _host: &str, _tls_verify: &TlsVerify) -> Result<(), Self::Error> {
        unimplemented!("ESP transport not yet implemented — use mock transport for host testing")
    }
}
