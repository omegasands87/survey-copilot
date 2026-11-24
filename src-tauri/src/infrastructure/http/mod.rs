use crate::domain::endpoint_validator::EndpointValidator;
use crate::domain::provider::{NormalizedError, NormalizedErrorCategory};
use std::collections::HashMap;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Representation of an HTTP response
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl HttpResponse {
    pub fn new(status: u16, body: impl Into<String>) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: body.into(),
        }
    }

    pub fn with_header(mut self, key: impl Into<String>, val: impl Into<String>) -> Self {
        self.headers.insert(key.into().to_lowercase(), val.into());
        self
    }
}

/// Abstract HTTP transport for AI provider requests.
/// Validates URL, resolves and pins DNS to public addresses, disables proxy bypasses,
/// and blocks redirects that cross the approved origin.
pub trait HttpTransport: Send + Sync {
    fn post_json(
        &self,
        url: &str,
        headers: &[(&str, &str)],
        body_json: &serde_json::Value,
        timeout_ms: u64,
    ) -> Result<HttpResponse, NormalizedError>;

    fn get(
        &self,
        url: &str,
        headers: &[(&str, &str)],
        timeout_ms: u64,
    ) -> Result<HttpResponse, NormalizedError>;
}

/// Production HTTP transport. A client is built per request so the validated DNS
/// answers can be pinned and cannot be replaced by a second lookup at connect time.
pub struct DefaultHttpTransport;

impl DefaultHttpTransport {
    pub fn new() -> Self {
        Self
    }

    fn invalid_endpoint(message: impl Into<String>) -> NormalizedError {
        NormalizedError::new(NormalizedErrorCategory::InvalidRequest, message)
    }

    fn build_client_for_url(url: &str) -> Result<(String, reqwest::Client), NormalizedError> {
        let validated_url =
            EndpointValidator::validate_url(url).map_err(|e| Self::invalid_endpoint(e.message))?;
        let parsed = url::Url::parse(&validated_url)
            .map_err(|_| Self::invalid_endpoint("URL endpoint tidak valid."))?;
        let port = parsed
            .port_or_known_default()
            .ok_or_else(|| Self::invalid_endpoint("Port endpoint tidak valid."))?;

        let (resolution_host, literal_ip) = match parsed.host() {
            Some(url::Host::Domain(domain)) => (domain.to_string(), None),
            Some(url::Host::Ipv4(ip)) => (ip.to_string(), Some(std::net::IpAddr::V4(ip))),
            Some(url::Host::Ipv6(ip)) => (ip.to_string(), Some(std::net::IpAddr::V6(ip))),
            None => return Err(Self::invalid_endpoint("Host endpoint tidak valid.")),
        };
        let allow_loopback =
            parsed.scheme() == "http" && EndpointValidator::is_local_host(&resolution_host);

        let addresses: Vec<SocketAddr> = if let Some(ip) = literal_ip {
            vec![SocketAddr::new(ip, 0)]
        } else {
            (resolution_host.as_str(), port)
                .to_socket_addrs()
                .map_err(|_| Self::invalid_endpoint("DNS endpoint tidak dapat di-resolve."))?
                .map(|address| SocketAddr::new(address.ip(), 0))
                .collect()
        };

        if addresses.is_empty() {
            return Err(Self::invalid_endpoint(
                "DNS endpoint tidak mengembalikan alamat tujuan.",
            ));
        }

        if addresses
            .iter()
            .any(|address| !EndpointValidator::is_allowed_ip(address.ip(), allow_loopback))
        {
            return Err(Self::invalid_endpoint(
                "DNS endpoint mengarah ke alamat private, reserved, loopback, atau link-local; permintaan diblokir.",
            ));
        }

        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .no_proxy()
            .resolve_to_addrs(&resolution_host, &addresses)
            .build()
            .map_err(|_| Self::invalid_endpoint("HTTP transport gagal diinisialisasi."))?;

        Ok((validated_url, client))
    }

    async fn execute_with_redirect_check(
        &self,
        original_url: &str,
        request_builder: reqwest::RequestBuilder,
    ) -> Result<HttpResponse, NormalizedError> {
        let validated_url = EndpointValidator::validate_url(original_url)
            .map_err(|e| Self::invalid_endpoint(e.message))?;

        let response = request_builder.send().await.map_err(|e| {
            if e.is_timeout() {
                NormalizedError::new(
                    NormalizedErrorCategory::Timeout,
                    "Permintaan ke provider AI melampaui batas waktu (timeout).",
                )
            } else if e.is_connect() {
                NormalizedError::new(
                    NormalizedErrorCategory::NetworkError,
                    format!("Koneksi ke endpoint gagal: {e}"),
                )
            } else {
                NormalizedError::new(
                    NormalizedErrorCategory::NetworkError,
                    format!("Kesalahan jaringan saat menghubungi provider: {e}"),
                )
            }
        })?;

        let status = response.status().as_u16();

        if (300..400).contains(&status) {
            if let Some(location) = response.headers().get(reqwest::header::LOCATION) {
                if let Ok(loc_str) = location.to_str() {
                    if !EndpointValidator::is_safe_redirect(&validated_url, loc_str) {
                        return Err(NormalizedError::new(
                            NormalizedErrorCategory::InvalidRequest,
                            "Pengalihan (redirect) ke origin berbeda dicegah untuk melindungi kredensial API.",
                        ));
                    }
                }
            }
        }

        let mut headers = HashMap::new();
        for (key, value) in response.headers() {
            if let Ok(value) = value.to_str() {
                headers.insert(key.as_str().to_lowercase(), value.to_string());
            }
        }

        let body = response.text().await.map_err(|e| {
            NormalizedError::new(
                NormalizedErrorCategory::ResponseParseError,
                format!("Gagal membaca isi respon provider: {e}"),
            )
        })?;

        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    }
}

impl Default for DefaultHttpTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpTransport for DefaultHttpTransport {
    fn post_json(
        &self,
        url: &str,
        headers: &[(&str, &str)],
        body_json: &serde_json::Value,
        timeout_ms: u64,
    ) -> Result<HttpResponse, NormalizedError> {
        let (validated_url, client) = Self::build_client_for_url(url)?;
        let mut request = client
            .post(&validated_url)
            .timeout(Duration::from_millis(timeout_ms))
            .json(body_json);

        for (key, value) in headers {
            request = request.header(*key, *value);
        }

        tauri::async_runtime::block_on(self.execute_with_redirect_check(&validated_url, request))
    }

    fn get(
        &self,
        url: &str,
        headers: &[(&str, &str)],
        timeout_ms: u64,
    ) -> Result<HttpResponse, NormalizedError> {
        let (validated_url, client) = Self::build_client_for_url(url)?;
        let mut request = client
            .get(&validated_url)
            .timeout(Duration::from_millis(timeout_ms));

        for (key, value) in headers {
            request = request.header(*key, *value);
        }

        tauri::async_runtime::block_on(self.execute_with_redirect_check(&validated_url, request))
    }
}

pub type RecordedMockRequest = (String, Vec<(String, String)>, Option<serde_json::Value>);

/// Test Mock Transport allowing deterministic responses without network calls
#[derive(Clone, Default)]
pub struct MockHttpTransport {
    responses: Arc<Mutex<Vec<HttpResponse>>>,
    recorded_requests: Arc<Mutex<Vec<RecordedMockRequest>>>,
}

impl MockHttpTransport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enqueue_response(&self, response: HttpResponse) {
        let mut responses = self.responses.lock().unwrap();
        responses.push(response);
    }

    pub fn get_recorded_requests(&self) -> Vec<RecordedMockRequest> {
        self.recorded_requests.lock().unwrap().clone()
    }
}

impl HttpTransport for MockHttpTransport {
    fn post_json(
        &self,
        url: &str,
        headers: &[(&str, &str)],
        body_json: &serde_json::Value,
        _timeout_ms: u64,
    ) -> Result<HttpResponse, NormalizedError> {
        EndpointValidator::validate_url(url).map_err(|e| {
            NormalizedError::new(NormalizedErrorCategory::InvalidRequest, e.message)
        })?;

        let header_vec = headers
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();

        self.recorded_requests.lock().unwrap().push((
            url.to_string(),
            header_vec,
            Some(body_json.clone()),
        ));

        let mut responses = self.responses.lock().unwrap();
        if responses.is_empty() {
            Err(NormalizedError::new(
                NormalizedErrorCategory::NetworkError,
                "Mock response queue empty",
            ))
        } else {
            Ok(responses.remove(0))
        }
    }

    fn get(
        &self,
        url: &str,
        headers: &[(&str, &str)],
        _timeout_ms: u64,
    ) -> Result<HttpResponse, NormalizedError> {
        EndpointValidator::validate_url(url).map_err(|e| {
            NormalizedError::new(NormalizedErrorCategory::InvalidRequest, e.message)
        })?;

        let header_vec = headers
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();

        self.recorded_requests
            .lock()
            .unwrap()
            .push((url.to_string(), header_vec, None));

        let mut responses = self.responses.lock().unwrap();
        if responses.is_empty() {
            Err(NormalizedError::new(
                NormalizedErrorCategory::NetworkError,
                "Mock response queue empty",
            ))
        } else {
            Ok(responses.remove(0))
        }
    }
}
