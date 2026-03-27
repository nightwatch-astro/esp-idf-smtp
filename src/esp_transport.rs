//! ESP-IDF transport implementation using `esp_tls` and `std::net::TcpStream`.

use crate::config::{SmtpConfig, TlsMode, TlsVerify};
use crate::transport::SmtpTransport;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

/// ESP-IDF transport using `esp_tls` for TLS and `std::net::TcpStream` for plaintext.
///
/// Supports three connection modes:
/// - **Implicit TLS**: TLS from connection start via `esp_tls`
/// - **STARTTLS**: Plaintext `TcpStream` upgraded to TLS via `esp_tls`
/// - **Plain**: `TcpStream` only (no TLS)
pub struct EspTransport {
    inner: TransportInner,
}

enum TransportInner {
    /// Plaintext TCP stream (for Plain mode or pre-STARTTLS).
    Plain(TcpStream),
    /// TLS connection via esp_tls (for Implicit TLS or post-STARTTLS).
    Tls(EspTlsConnection),
}

/// Wrapper around esp_tls handle.
struct EspTlsConnection {
    tls: *mut esp_idf_svc::sys::esp_tls_t,
}

// SAFETY: esp_tls_t is used single-threaded within an SMTP session.
unsafe impl Send for EspTlsConnection {}
unsafe impl Sync for EspTlsConnection {}

impl Drop for EspTlsConnection {
    fn drop(&mut self) {
        if !self.tls.is_null() {
            unsafe {
                esp_idf_svc::sys::esp_tls_conn_destroy(self.tls);
            }
        }
    }
}

impl EspTransport {
    /// Connect using the configured TLS mode.
    pub fn connect(config: &SmtpConfig) -> Result<Self, io::Error> {
        match config.tls_mode {
            TlsMode::ImplicitTls => Self::connect_tls(config),
            TlsMode::StartTls | TlsMode::Plain => Self::connect_plain(config),
        }
    }

    /// Connect with TLS from the start (implicit TLS, typically port 465).
    fn connect_tls(config: &SmtpConfig) -> Result<Self, io::Error> {
        let tls_cfg = build_tls_cfg(&config.tls_verify);

        let tls = unsafe { esp_idf_svc::sys::esp_tls_init() };
        if tls.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "failed to initialize esp_tls",
            ));
        }

        let host_cstr = std::ffi::CString::new(config.host.as_str())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

        let ret = unsafe {
            esp_idf_svc::sys::esp_tls_conn_new_sync(
                host_cstr.as_ptr(),
                host_cstr.as_bytes().len() as i32,
                config.port as i32,
                &tls_cfg,
                tls,
            )
        };

        if ret != 1 {
            unsafe {
                esp_idf_svc::sys::esp_tls_conn_destroy(tls);
            }
            return Err(io::Error::new(
                io::ErrorKind::ConnectionRefused,
                format!(
                    "esp_tls connect to {}:{} failed (ret={})",
                    config.host, config.port, ret
                ),
            ));
        }

        log::debug!("Connected to {}:{} (implicit TLS)", config.host, config.port);

        Ok(Self {
            inner: TransportInner::Tls(EspTlsConnection { tls }),
        })
    }

    /// Connect with plaintext TCP (for STARTTLS or plain mode).
    fn connect_plain(config: &SmtpConfig) -> Result<Self, io::Error> {
        let addr = format!("{}:{}", config.host, config.port);
        let timeout = Duration::from_millis(config.timeout_ms as u64);

        let stream = TcpStream::connect_timeout(
            &addr
                .parse()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?,
            timeout,
        )?;

        stream.set_read_timeout(Some(timeout))?;
        stream.set_write_timeout(Some(timeout))?;

        log::debug!("Connected to {} (plaintext)", addr);

        Ok(Self {
            inner: TransportInner::Plain(stream),
        })
    }
}

impl SmtpTransport for EspTransport {
    type Error = io::Error;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        match &mut self.inner {
            TransportInner::Plain(stream) => stream.read(buf),
            TransportInner::Tls(conn) => {
                let ret =
                    unsafe { esp_idf_svc::sys::esp_tls_conn_read(conn.tls, buf.as_mut_ptr() as *mut _, buf.len()) };

                if ret > 0 {
                    Ok(ret as usize)
                } else if ret == 0 {
                    Ok(0) // EOF
                } else {
                    Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("esp_tls_conn_read failed: {}", ret),
                    ))
                }
            }
        }
    }

    fn write_all(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        match &mut self.inner {
            TransportInner::Plain(stream) => stream.write_all(data),
            TransportInner::Tls(conn) => {
                let mut written = 0;
                while written < data.len() {
                    let ret = unsafe {
                        esp_idf_svc::sys::esp_tls_conn_write(
                            conn.tls,
                            data[written..].as_ptr() as *const _,
                            data.len() - written,
                        )
                    };

                    if ret > 0 {
                        written += ret as usize;
                    } else {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            format!("esp_tls_conn_write failed: {}", ret),
                        ));
                    }
                }
                Ok(())
            }
        }
    }

    fn upgrade_tls(&mut self, host: &str, tls_verify: &TlsVerify) -> Result<(), Self::Error> {
        // Extract the TcpStream fd for TLS upgrade
        let stream = match &self.inner {
            TransportInner::Plain(stream) => stream,
            TransportInner::Tls(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "already using TLS — cannot upgrade",
                ));
            }
        };

        // Get the raw fd from the TcpStream
        use std::os::fd::AsRawFd;
        let fd = stream.as_raw_fd();

        let tls_cfg = build_tls_cfg(tls_verify);

        let tls = unsafe { esp_idf_svc::sys::esp_tls_init() };
        if tls.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "failed to initialize esp_tls for STARTTLS upgrade",
            ));
        }

        // Set the existing socket fd
        unsafe {
            // esp_tls supports upgrading an existing connection
            (*tls).sockfd = fd;
        }

        let host_cstr = std::ffi::CString::new(host)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

        let ret = unsafe {
            esp_idf_svc::sys::esp_tls_conn_new_sync(
                host_cstr.as_ptr(),
                host_cstr.as_bytes().len() as i32,
                0, // port not needed for upgrade
                &tls_cfg,
                tls,
            )
        };

        if ret != 1 {
            unsafe {
                esp_idf_svc::sys::esp_tls_conn_destroy(tls);
            }
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("STARTTLS upgrade failed for {}", host),
            ));
        }

        log::debug!("STARTTLS upgrade complete for {}", host);

        // Replace inner with TLS connection
        // Note: we intentionally leak the TcpStream fd since esp_tls now owns it
        self.inner = TransportInner::Tls(EspTlsConnection { tls });
        Ok(())
    }
}

/// Build esp_tls_cfg_t from TlsVerify settings.
fn build_tls_cfg(tls_verify: &TlsVerify) -> esp_idf_svc::sys::esp_tls_cfg_t {
    let mut cfg: esp_idf_svc::sys::esp_tls_cfg_t = unsafe { std::mem::zeroed() };

    match tls_verify {
        TlsVerify::Verify => {
            cfg.use_global_ca_store = true;
        }
        TlsVerify::SkipVerify => {
            cfg.skip_common_name = true;
        }
        TlsVerify::CustomCa(pem) => {
            cfg.cacert_buf = pem.as_ptr();
            cfg.cacert_bytes = pem.len() as u32;
        }
    }

    cfg
}
