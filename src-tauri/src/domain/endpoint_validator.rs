use crate::domain::error::AppError;

/// Validates and normalizes custom endpoint URLs for AI providers.
/// Enforces HTTPS by default and protects against SSRF (Server-Side Request Forgery).
pub struct EndpointValidator;

impl EndpointValidator {
    /// Validates an endpoint URL according to security invariants:
    /// 1. Must be a well-formed HTTP/HTTPS URL.
    /// 2. Scheme must be HTTPS by default.
    /// 3. Plain HTTP is strictly forbidden except for localhost / 127.0.0.1 / [::1] (AT-037).
    /// 4. Disallows dangerous schemes (`file:`, `ftp:`, `javascript:`, `data:`, etc.).
    /// 5. Disallows cloud metadata endpoints (e.g. AWS/GCP 169.254.169.254, metadata.google.internal).
    /// 6. Strips trailing slashes to ensure consistent URL concatenation.
    pub fn validate_url(url_str: &str) -> Result<String, AppError> {
        let trimmed = url_str.trim();
        if trimmed.is_empty() {
            return Err(AppError::validation(
                "baseUrl",
                "URL endpoint tidak boleh kosong.",
            ));
        }

        // Scheme check
        let lower = trimmed.to_lowercase();
        let (scheme, rest) = if let Some(idx) = lower.find("://") {
            (&lower[..idx], &trimmed[idx + 3..])
        } else {
            return Err(AppError::validation(
                "baseUrl",
                "Format URL tidak valid. URL harus diawali dengan skema 'https://' atau 'http://'.",
            ));
        };

        if scheme != "http" && scheme != "https" {
            return Err(AppError::validation(
                "baseUrl",
                format!(
                    "Skema URL '{scheme}://' tidak diizinkan. Hanya 'https://' yang diperbolehkan."
                ),
            ));
        }

        if rest.is_empty() {
            return Err(AppError::validation(
                "baseUrl",
                "Host pada URL endpoint tidak boleh kosong.",
            ));
        }

        // Extract host and optional port/path
        let host_port_part = rest.split(['/', '?', '#']).next().unwrap_or("").trim();
        if host_port_part.is_empty() {
            return Err(AppError::validation(
                "baseUrl",
                "Host pada URL endpoint tidak valid.",
            ));
        }

        // Separate host from port
        let host = if host_port_part.starts_with('[') {
            // IPv6
            if let Some(end_bracket) = host_port_part.find(']') {
                &host_port_part[1..end_bracket]
            } else {
                return Err(AppError::validation(
                    "baseUrl",
                    "Format alamat IPv6 tidak valid.",
                ));
            }
        } else if let Some(colon_idx) = host_port_part.find(':') {
            &host_port_part[..colon_idx]
        } else {
            host_port_part
        };

        let host_lower = host.to_lowercase();

        // SSRF Check: Reject cloud instance metadata IPs and hostnames
        if Self::is_blocked_metadata_host(&host_lower) {
            return Err(AppError::validation(
                "baseUrl",
                "Akses ke endpoint metadata cloud atau alamat internal dilarang untuk keamanan.",
            ));
        }

        // AT-037 Check: HTTP endpoint outside localhost rejected
        let is_localhost = host_lower == "localhost"
            || host_lower == "127.0.0.1"
            || host_lower == "::1"
            || host_lower == "0:0:0:0:0:0:0:1";

        if scheme == "http" && !is_localhost {
            return Err(AppError::validation(
                "baseUrl",
                "Endpoint HTTP hanya diperbolehkan untuk pengujian lokal (localhost / 127.0.0.1). Endpoint eksternal wajib menggunakan HTTPS.",
            ));
        }

        // Normalize: trim trailing slashes
        let mut normalized = trimmed.to_string();
        while normalized.ends_with('/') {
            normalized.pop();
        }

        Ok(normalized)
    }

    /// Checks if a host is an internal cloud metadata address
    fn is_blocked_metadata_host(host: &str) -> bool {
        if host == "169.254.169.254"
            || host == "metadata.google.internal"
            || host == "100.100.100.200"
            || host == "metadata.tencentyun.com"
        {
            return true;
        }

        // Reject link-local IPv4 range 169.254.0.0/16
        if host.starts_with("169.254.") {
            return true;
        }

        false
    }

    /// AT-038: Check if redirect target is a safe host (must not cross to an unapproved different origin)
    pub fn is_safe_redirect(original_url: &str, redirect_target: &str) -> bool {
        let orig_clean = original_url.trim();
        let target_clean = redirect_target.trim();

        // Target must still pass endpoint validation
        if Self::validate_url(target_clean).is_err() {
            return false;
        }

        // Extract hosts
        fn extract_host(u: &str) -> Option<String> {
            let lower = u.to_lowercase();
            let idx = lower.find("://")?;
            let rest = &lower[idx + 3..];
            let host_port = rest.split(['/', '?', '#']).next()?;
            let host = if host_port.starts_with('[') {
                if let Some(end) = host_port.find(']') {
                    &host_port[1..end]
                } else {
                    host_port
                }
            } else if let Some(colon) = host_port.find(':') {
                &host_port[..colon]
            } else {
                host_port
            };
            Some(host.to_string())
        }

        let orig_host = extract_host(orig_clean);
        let target_host = extract_host(target_clean);

        match (orig_host, target_host) {
            (Some(o), Some(t)) => o == t,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_https_url_passes() {
        let url = "https://generativelanguage.googleapis.com";
        let res = EndpointValidator::validate_url(url);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), "https://generativelanguage.googleapis.com");
    }

    #[test]
    fn test_trailing_slashes_are_stripped() {
        let url = "https://api.openai.com/v1///";
        let res = EndpointValidator::validate_url(url).unwrap();
        assert_eq!(res, "https://api.openai.com/v1");
    }

    #[test]
    fn test_at_037_http_localhost_is_allowed() {
        let url1 = "http://localhost:11434";
        assert!(EndpointValidator::validate_url(url1).is_ok());

        let url2 = "http://127.0.0.1:8080/v1";
        assert!(EndpointValidator::validate_url(url2).is_ok());

        let url3 = "http://[::1]:8000";
        assert!(EndpointValidator::validate_url(url3).is_ok());
    }

    #[test]
    fn test_at_037_http_outside_localhost_is_rejected() {
        let url = "http://api.example.com";
        let res = EndpointValidator::validate_url(url);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.message.contains("Endpoint HTTP hanya diperbolehkan"));
    }

    #[test]
    fn test_dangerous_schemes_are_rejected() {
        assert!(EndpointValidator::validate_url("file:///etc/passwd").is_err());
        assert!(EndpointValidator::validate_url("javascript:alert(1)").is_err());
        assert!(EndpointValidator::validate_url("ftp://ftp.example.com").is_err());
        assert!(EndpointValidator::validate_url("data:text/html,test").is_err());
    }

    #[test]
    fn test_ssrf_metadata_endpoints_are_rejected() {
        assert!(
            EndpointValidator::validate_url("https://169.254.169.254/latest/meta-data").is_err()
        );
        assert!(EndpointValidator::validate_url(
            "https://metadata.google.internal/computeMetadata/v1"
        )
        .is_err());
        assert!(EndpointValidator::validate_url("https://169.254.1.1").is_err());
    }

    #[test]
    fn test_at_038_redirect_host_isolation() {
        let orig = "https://generativelanguage.googleapis.com/v1beta/models";
        // Same host redirect: allowed
        assert!(EndpointValidator::is_safe_redirect(
            orig,
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash"
        ));
        // Cross host redirect: strictly rejected
        assert!(!EndpointValidator::is_safe_redirect(
            orig,
            "https://evil-attacker.com/steal-key"
        ));
        assert!(!EndpointValidator::is_safe_redirect(
            orig,
            "http://another-host.com/v1"
        ));
    }
}
