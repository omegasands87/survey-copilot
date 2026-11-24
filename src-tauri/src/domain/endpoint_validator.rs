use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use url::Url;

use crate::domain::error::AppError;

/// Validates custom provider endpoints and prevents requests to local/private network targets.
pub struct EndpointValidator;

impl EndpointValidator {
    pub fn validate_url(url_str: &str) -> Result<String, AppError> {
        let trimmed = url_str.trim();
        if trimmed.is_empty() {
            return Err(AppError::validation("baseUrl", "URL endpoint tidak boleh kosong."));
        }

        let parsed = Url::parse(trimmed).map_err(|_| {
            AppError::validation(
                "baseUrl",
                "Format URL tidak valid. Gunakan URL HTTP/HTTPS yang lengkap.",
            )
        })?;

        if parsed.scheme() != "http" && parsed.scheme() != "https" {
            return Err(AppError::validation(
                "baseUrl",
                "Hanya skema HTTPS yang diizinkan, kecuali HTTP untuk localhost.",
            ));
        }

        if parsed.host_str().is_none() || parsed.host_str().unwrap_or_default().is_empty() {
            return Err(AppError::validation("baseUrl", "Host URL endpoint tidak valid."));
        }

        if !parsed.username().is_empty() || parsed.password().is_some() {
            return Err(AppError::validation(
                "baseUrl",
                "URL endpoint tidak boleh menyimpan username atau password; gunakan konfigurasi kredensial terpisah.",
            ));
        }

        if parsed.fragment().is_some() {
            return Err(AppError::validation(
                "baseUrl",
                "URL endpoint tidak boleh memiliki fragment (#...).",
            ));
        }

        let host = parsed.host_str().unwrap_or_default().trim_end_matches('.').to_ascii_lowercase();
        if Self::is_blocked_metadata_host(&host) {
            return Err(AppError::validation(
                "baseUrl",
                "Akses ke endpoint metadata cloud atau hostname jaringan lokal dilarang.",
            ));
        }

        let local_host = Self::is_local_host(&host);
        if parsed.scheme() == "http" && !local_host {
            return Err(AppError::validation(
                "baseUrl",
                "Endpoint HTTP hanya diperbolehkan untuk localhost / loopback. Endpoint lainnya wajib menggunakan HTTPS.",
            ));
        }

        // Literal IP dapat diputuskan pada tahap validasi konfigurasi.
        // Hostname akan di-resolve dan dipin ke IP yang telah diperiksa oleh HttpTransport.
        if let Ok(ip) = host.parse::<IpAddr>() {
            if !Self::is_allowed_ip(ip, local_host) {
                return Err(AppError::validation(
                    "baseUrl",
                    "Endpoint menuju alamat IP private, reserved, loopback, atau link-local tidak diizinkan.",
                ));
            }
        }

        let normalized = parsed.as_str().trim_end_matches('/').to_string();
        Ok(normalized)
    }

    pub fn is_local_host(host: &str) -> bool {
        let normalized = host.trim_matches(['[', ']']).trim_end_matches('.').to_ascii_lowercase();
        normalized == "localhost"
            || normalized.parse::<IpAddr>().map(|ip| ip.is_loopback()).unwrap_or(false)
    }

    /// True only for globally routable destinations, except explicitly allowed local test endpoints.
    pub fn is_allowed_ip(ip: IpAddr, allow_loopback: bool) -> bool {
        match ip {
            IpAddr::V4(ipv4) => Self::is_allowed_ipv4(ipv4, allow_loopback),
            IpAddr::V6(ipv6) => Self::is_allowed_ipv6(ipv6, allow_loopback),
        }
    }

    fn is_allowed_ipv4(ip: Ipv4Addr, allow_loopback: bool) -> bool {
        if allow_loopback && ip.is_loopback() {
            return true;
        }

        if ip.is_private()
            || ip.is_loopback()
            || ip.is_link_local()
            || ip.is_unspecified()
            || ip.is_multicast()
            || ip.is_broadcast()
        {
            return false;
        }

        let [a, b, c, _d] = ip.octets();

        // Shared address space, protocol assignments, documentation, benchmarking,
        // deprecated relay, and reserved/future-use blocks are not public destinations.
        let shared = a == 100 && (64..=127).contains(&b);
        let protocol_assignment = a == 192 && b == 0 && c == 0;
        let documentation = (a == 192 && b == 0 && c == 2)
            || (a == 198 && b == 51 && c == 100)
            || (a == 203 && b == 0 && c == 113);
        let benchmark = a == 198 && (b == 18 || b == 19);
        let deprecated_relay = a == 192 && b == 88 && c == 99;
        let reserved = a >= 240;

        !(shared || protocol_assignment || documentation || benchmark || deprecated_relay || reserved)
    }

    fn is_allowed_ipv6(ip: Ipv6Addr, allow_loopback: bool) -> bool {
        if allow_loopback && ip.is_loopback() {
            return true;
        }

        if ip.is_loopback()
            || ip.is_unspecified()
            || ip.is_multicast()
            || ip.is_unicast_link_local()
        {
            return false;
        }

        let octets = ip.octets();
        // Reject IPv4-mapped IPv6 unless the embedded IPv4 is itself allowed.
        if octets[..10] == [0; 10] && octets[10..12] == [0xff, 0xff] {
            return Self::is_allowed_ipv4(
                Ipv4Addr::new(octets[12], octets[13], octets[14], octets[15]),
                allow_loopback,
            );
        }

        let segments = ip.segments();
        let unique_local = segments[0] & 0xfe00 == 0xfc00; // fc00::/7
        let global_unicast = segments[0] & 0xe000 == 0x2000; // 2000::/3
        let documentation = segments[0] == 0x2001 && segments[1] == 0x0db8;

        global_unicast && !unique_local && !documentation
    }

    fn is_blocked_metadata_host(host: &str) -> bool {
        host == "metadata.google.internal"
            || host == "metadata.tencentyun.com"
            || host == "metadata.azure.internal"
            || host == "instance-data"
            || host.ends_with(".localhost")
            || host.ends_with(".local")
            || host.ends_with(".internal")
            || host.ends_with(".test")
            || host.ends_with(".invalid")
    }

    /// Redirect hanya dianggap aman jika tetap berada di origin yang sama.
    /// Relative Location di-resolve terhadap URL asal.
    pub fn is_safe_redirect(original_url: &str, redirect_target: &str) -> bool {
        let Ok(original) = Url::parse(original_url.trim()) else {
            return false;
        };
        let Ok(target) = original.join(redirect_target.trim()) else {
            return false;
        };

        if Self::validate_url(target.as_str()).is_err() {
            return false;
        }

        original.origin() == target.origin()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_https_url_passes_and_trailing_slashes_are_stripped() {
        let normalized = EndpointValidator::validate_url("https://api.openai.com/v1///").unwrap();
        assert_eq!(normalized, "https://api.openai.com/v1");
    }

    #[test]
    fn test_local_http_is_allowed_only_for_loopback() {
        assert!(EndpointValidator::validate_url("http://localhost:11434").is_ok());
        assert!(EndpointValidator::validate_url("http://127.0.0.1:8080/v1").is_ok());
        assert!(EndpointValidator::validate_url("http://[::1]:8000").is_ok());
        assert!(EndpointValidator::validate_url("http://192.168.1.8:8080").is_err());
        assert!(EndpointValidator::validate_url("http://api.example.com").is_err());
    }

    #[test]
    fn test_credentials_and_fragments_in_url_are_rejected() {
        assert!(EndpointValidator::validate_url("https://user:pass@example.com/v1").is_err());
        assert!(EndpointValidator::validate_url("https://example.com/v1#fragment").is_err());
    }

    #[test]
    fn test_private_reserved_and_metadata_addresses_are_rejected() {
        for url in [
            "https://10.0.0.2",
            "https://172.16.0.1",
            "https://192.168.1.2",
            "https://100.64.0.1",
            "https://169.254.169.254/latest/meta-data",
            "https://192.0.2.1",
            "https://198.18.0.1",
            "https://metadata.google.internal",
            "https://printer.local",
            "https://service.internal",
            "https://[fc00::1]",
            "https://[fe80::1]",
            "https://[2001:db8::1]",
        ] {
            assert!(EndpointValidator::validate_url(url).is_err(), "must reject {url}");
        }
    }

    #[test]
    fn test_public_literal_ips_are_allowed() {
        assert!(EndpointValidator::validate_url("https://8.8.8.8/dns-query").is_ok());
        assert!(EndpointValidator::validate_url("https://[2606:4700:4700::1111]").is_ok());
    }

    #[test]
    fn test_redirect_must_preserve_exact_origin() {
        let original = "https://api.example.com:443/v1/models";
        assert!(EndpointValidator::is_safe_redirect(original, "/v1/models/next"));
        assert!(EndpointValidator::is_safe_redirect(original, "https://api.example.com/v1/next"));
        assert!(!EndpointValidator::is_safe_redirect(original, "http://api.example.com/v1/next"));
        assert!(!EndpointValidator::is_safe_redirect(original, "https://api.example.com:8443/v1/next"));
        assert!(!EndpointValidator::is_safe_redirect(original, "https://evil.example.net/steal"));
        assert!(!EndpointValidator::is_safe_redirect(original, "https://api.example.com@evil.example.net/steal"));
    }
}
