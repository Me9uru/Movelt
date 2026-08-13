use std::time::Duration;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use reqwest::{Client, StatusCode};
use serde::de::DeserializeOwned;
use url::Url;

use crate::error::NovelError;

pub(super) struct Wenku8Client {
    http: Client,
    base_url: Url,
}

impl Wenku8Client {
    pub(super) fn new(base_url: &str) -> Result<Self, NovelError> {
        let base_url = Url::parse(&format!("{}/", base_url.trim_end_matches('/')))
            .map_err(|error| NovelError::configuration(error.to_string()))?;
        if !matches!(base_url.scheme(), "http" | "https") || base_url.host_str().is_none() {
            return Err(NovelError::configuration(
                "Wenku8 API base URL must be an absolute http(s) URL",
            ));
        }
        let http = Client::builder()
            .connect_timeout(Duration::from_secs(8))
            .timeout(Duration::from_secs(45))
            .build()
            .map_err(|error| NovelError::configuration(error.to_string()))?;
        Ok(Self { http, base_url })
    }

    pub(super) fn base_url(&self) -> String {
        self.base_url.as_str().trim_end_matches('/').into()
    }

    pub(super) fn url(&self, path: &str, query: &[(&str, String)]) -> Result<Url, NovelError> {
        let mut url = self
            .base_url
            .join(path.trim_start_matches('/'))
            .map_err(|error| NovelError::invalid_input(error.to_string()))?;
        if !query.is_empty() {
            url.query_pairs_mut()
                .extend_pairs(query.iter().map(|(key, value)| (*key, value.as_str())));
        }
        Ok(url)
    }

    pub(super) async fn json<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<T, NovelError> {
        let bytes = self.request(path, query).await?;
        serde_json::from_slice(&bytes).map_err(|error| NovelError::Parse(error.to_string()))
    }

    pub(super) async fn text(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<String, NovelError> {
        let bytes = self.request(path, query).await?;
        String::from_utf8(bytes.to_vec()).map_err(|error| NovelError::Parse(error.to_string()))
    }

    pub(super) async fn image_data_url(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<String, NovelError> {
        let url = self.url(path, query)?;
        for attempt in 0..=1 {
            let response = self.http.get(url.clone()).send().await.map_err(|error| {
                NovelError::Upstream(format!("{} ({})", error, self.base_url()))
            })?;
            let status = response.status();
            let content_type = response
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.split(';').next())
                .map(str::trim)
                .unwrap_or("")
                .to_owned();
            let body = response
                .bytes()
                .await
                .map_err(|error| NovelError::Upstream(error.to_string()))?;
            if status.is_success() {
                if !matches!(
                    content_type.as_str(),
                    "image/jpeg" | "image/png" | "image/gif" | "image/webp" | "image/avif"
                ) {
                    return Err(NovelError::Parse(format!(
                        "cover response is not a supported image ({content_type})"
                    )));
                }
                if body.len() > 10 * 1024 * 1024 {
                    return Err(NovelError::Parse("cover image exceeds 10 MiB".into()));
                }
                return Ok(format!(
                    "data:{content_type};base64,{}",
                    STANDARD.encode(&body)
                ));
            }
            if matches!(
                status,
                StatusCode::TOO_MANY_REQUESTS | StatusCode::BAD_GATEWAY
            ) && attempt == 0
            {
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
            return Err(map_http_error(status, &body));
        }
        Err(NovelError::Internal)
    }

    async fn request(&self, path: &str, query: &[(&str, String)]) -> Result<Vec<u8>, NovelError> {
        let url = self.url(path, query)?;
        for attempt in 0..=1 {
            let response = self.http.get(url.clone()).send().await.map_err(|error| {
                NovelError::Upstream(format!("{} ({})", error, self.base_url()))
            })?;
            let status = response.status();
            let body = response
                .bytes()
                .await
                .map_err(|error| NovelError::Upstream(error.to_string()))?;
            if status.is_success() {
                return Ok(body.to_vec());
            }
            if matches!(
                status,
                StatusCode::TOO_MANY_REQUESTS | StatusCode::BAD_GATEWAY
            ) && attempt == 0
            {
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
            return Err(map_http_error(status, &body));
        }
        Err(NovelError::Internal)
    }
}

fn map_http_error(status: StatusCode, body: &[u8]) -> NovelError {
    if status == StatusCode::UNAUTHORIZED {
        return NovelError::NotLoggedIn;
    }
    let message = serde_json::from_slice::<serde_json::Value>(body)
        .ok()
        .and_then(|value| value.get("detail")?.as_str().map(str::to_owned))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| format!("HTTP {status}"));
    match status {
        StatusCode::NOT_FOUND => NovelError::NotFound,
        StatusCode::UNPROCESSABLE_ENTITY => NovelError::InvalidInput(message),
        StatusCode::TOO_MANY_REQUESTS => NovelError::RateLimited(message),
        _ => NovelError::Upstream(message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    #[test]
    fn query_values_are_percent_encoded() {
        let client = Wenku8Client::new("http://127.0.0.1:8000/api").unwrap();
        let url = client
            .url("search", &[("keyword", "无职 转生".into())])
            .unwrap();
        assert_eq!(url.path(), "/api/search");
        assert!(url
            .as_str()
            .contains("keyword=%E6%97%A0%E8%81%8C+%E8%BD%AC%E7%94%9F"));
    }

    #[test]
    fn retries_a_rate_limited_request_once() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            for (index, stream) in listener.incoming().take(2).enumerate() {
                let mut stream = stream.unwrap();
                let mut request = [0_u8; 1024];
                let _ = stream.read(&mut request).unwrap();
                let (status, body) = if index == 0 {
                    ("429 Too Many Requests", r#"{"detail":"slow down"}"#)
                } else {
                    ("200 OK", r#"{"logged_in":true}"#)
                };
                write!(
                    stream,
                    "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                )
                .unwrap();
            }
        });
        let client = Wenku8Client::new(&format!("http://{address}")).unwrap();
        let health: serde_json::Value =
            tauri::async_runtime::block_on(client.json("health", &[])).unwrap();
        assert_eq!(health["logged_in"], true);
        server.join().unwrap();
    }

    #[test]
    fn converts_supported_images_to_data_urls() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request).unwrap();
            let body = [0xff, 0xd8, 0xff, 0xd9];
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            )
            .unwrap();
            stream.write_all(&body).unwrap();
        });
        let client = Wenku8Client::new(&format!("http://{address}")).unwrap();
        let data_url = tauri::async_runtime::block_on(client.image_data_url("cover", &[])).unwrap();
        assert_eq!(data_url, "data:image/jpeg;base64,/9j/2Q==");
        server.join().unwrap();
    }

    #[test]
    fn maps_validation_errors_without_marking_them_retryable() {
        let error = map_http_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            br#"{"detail":"invalid page"}"#,
        );
        assert!(matches!(error, NovelError::InvalidInput(message) if message == "invalid page"));
    }
}
