use super::NovelError;

const DEFAULT_WENKU8_API_BASE_URL: &str = "http://127.0.0.1:8000/";

pub(crate) struct NovelConfig {
    pub(crate) wenku8_api_base_url: String,
}

impl NovelConfig {
    pub(crate) fn load() -> Result<Self, NovelError> {
        let base_url = std::env::var("MOVEL_WENKU8_API_BASE_URL")
            .unwrap_or_else(|_| DEFAULT_WENKU8_API_BASE_URL.into());
        let parsed = url::Url::parse(base_url.trim()).map_err(|error| {
            NovelError::configuration(format!("invalid Wenku8 API URL: {error}"))
        })?;
        if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
            return Err(NovelError::configuration(
                "Wenku8 API URL must be an absolute http(s) URL",
            ));
        }
        if !parsed.username().is_empty() || parsed.password().is_some() {
            return Err(NovelError::configuration(
                "Wenku8 API URL must not contain credentials",
            ));
        }
        if parsed.query().is_some() || parsed.fragment().is_some() {
            return Err(NovelError::configuration(
                "Wenku8 API URL must not contain a query or fragment",
            ));
        }
        Ok(Self {
            wenku8_api_base_url: parsed.as_str().trim_end_matches('/').into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_embedded_novel_configuration() {
        let config = NovelConfig::load().unwrap();
        assert!(!config.wenku8_api_base_url.is_empty());
    }
}
