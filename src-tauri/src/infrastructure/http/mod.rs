use crate::domain::endpoint_validator::EndpointValidator;
use crate::domain::provider::{NormalizedError, NormalizedErrorCategory};
use std::collections::HashMap;
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
/// Enforces SSRF validation and redirect isolation.
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

/// Production HTTP Transport using reqwest with SSRF and redirect protection.
pub struct DefaultHttpTransport {
    client: reqwest::Client,
}

impl DefaultHttpTransport {
    pub fn new() -> Self {
        // Enforce AT-038: Custom redirect policy that prevents credential forwarding across hosts
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self { client }
    }

    async fn execute_with_redirect_check(
        &self,
        original_url: &str,
        request_builder: reqwest::RequestBuilder,
    ) -> Result<HttpResponse, NormalizedError> {
        // 1. SSRF validation on the initial URL
        let validated_url = EndpointValidator::validate_url(original_url).map_err(|e| {
            NormalizedError::new(NormalizedErrorCategory::InvalidRequest, e.message)
        })?;

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

        // 2. Handle redirect status codes (301, 302, 307, 308) - AT-038
        if (300..400).contains(&status) {
            if let Some(location) = response.headers().get(reqwest::header::LOCATION) {
                if let Ok(loc_str) = location.to_str() {
                    if !EndpointValidator::is_safe_redirect(&validated_url, loc_str) {
                        return Err(NormalizedError::new(
                            NormalizedErrorCategory::InvalidRequest,
                            "Pengalihan (redirect) ke host berbeda dicegah untuk melindungi kredensial API.",
                        ));
                    }
                }
            }
        }

        let mut headers = HashMap::new();
        for (k, v) in response.headers() {
            if let Ok(val_str) = v.to_str() {
                headers.insert(k.as_str().to_lowercase(), val_str.to_string());
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
        let validated_url = EndpointValidator::validate_url(url).map_err(|e| {
            NormalizedError::new(NormalizedErrorCategory::InvalidRequest, e.message)
        })?;

        let mut req = self
            .client
            .post(&validated_url)
            .timeout(Duration::from_millis(timeout_ms))
            .json(body_json);

        for (k, v) in headers {
            req = req.header(*k, *v);
        }

        tauri::async_runtime::block_on(self.execute_with_redirect_check(&validated_url, req))
    }

    fn get(
        &self,
        url: &str,
        headers: &[(&str, &str)],
        timeout_ms: u64,
    ) -> Result<HttpResponse, NormalizedError> {
        let validated_url = EndpointValidator::validate_url(url).map_err(|e| {
            NormalizedError::new(NormalizedErrorCategory::InvalidRequest, e.message)
        })?;

        let mut req = self
            .client
            .get(&validated_url)
            .timeout(Duration::from_millis(timeout_ms));

        for (k, v) in headers {
            req = req.header(*k, *v);
        }

        tauri::async_runtime::block_on(self.execute_with_redirect_check(&validated_url, req))
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
        let mut resps = self.responses.lock().unwrap();
        resps.push(response);
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
        // Enforce SSRF validation even in mock transport
        EndpointValidator::validate_url(url).map_err(|e| {
            NormalizedError::new(NormalizedErrorCategory::InvalidRequest, e.message)
        })?;

        let header_vec = headers
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        self.recorded_requests.lock().unwrap().push((
            url.to_string(),
            header_vec,
            Some(body_json.clone()),
        ));

        let mut resps = self.responses.lock().unwrap();
        if resps.is_empty() {
            Err(NormalizedError::new(
                NormalizedErrorCategory::NetworkError,
                "Mock response queue empty",
            ))
        } else {
            Ok(resps.remove(0))
        }
    }

    fn get(
        &self,
        url: &str,
        headers: &[(&str, &str)],
        _timeout_ms: u64,
    ) -> Result<HttpResponse, NormalizedError> {
        // Enforce SSRF validation even in mock transport
        EndpointValidator::validate_url(url).map_err(|e| {
            NormalizedError::new(NormalizedErrorCategory::InvalidRequest, e.message)
        })?;

        let header_vec = headers
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        self.recorded_requests
            .lock()
            .unwrap()
            .push((url.to_string(), header_vec, None));

        let mut resps = self.responses.lock().unwrap();
        if resps.is_empty() {
            Err(NormalizedError::new(
                NormalizedErrorCategory::NetworkError,
                "Mock response queue empty",
            ))
        } else {
            Ok(resps.remove(0))
        }
    }
}
