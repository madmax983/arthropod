//! Web font loading and caching helpers.

use std::collections::HashMap;

/// Font source for web/runtime loading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FontSource {
    /// Font bytes already available in memory.
    Bytes(Vec<u8>),
    /// Fetch font bytes from a URL.
    Url(String),
}

/// In-memory cache for fetched font bytes, keyed by URL + optional version.
#[derive(Debug, Default, Clone)]
pub struct FontCache {
    entries: HashMap<String, Vec<u8>>,
}

impl FontCache {
    /// Insert bytes using a raw cache key (typically URL).
    pub fn insert<K: Into<String>>(&mut self, key: K, bytes: Vec<u8>) {
        self.entries.insert(key.into(), bytes);
    }

    /// Get cached bytes for a raw key.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&[u8]> {
        self.entries.get(key).map(Vec::as_slice)
    }

    /// Insert bytes for a URL + optional version key.
    pub fn insert_versioned(&mut self, url: &str, version: Option<&str>, bytes: Vec<u8>) {
        self.insert(versioned_cache_key(url, version), bytes);
    }

    /// Get bytes for a URL + optional version key.
    #[must_use]
    pub fn get_versioned(&self, url: &str, version: Option<&str>) -> Option<&[u8]> {
        self.get(&versioned_cache_key(url, version))
    }
}

/// Build a deterministic cache key from URL + optional version.
#[must_use]
pub fn versioned_cache_key(url: &str, version: Option<&str>) -> String {
    match version {
        Some(version) if !version.is_empty() => format!("{url}#v={version}"),
        _ => url.to_string(),
    }
}

/// Load font bytes from a source.
pub async fn load_font_source(
    source: FontSource,
    cache: &mut FontCache,
) -> Result<Vec<u8>, WebFontLoadError> {
    match source {
        FontSource::Bytes(bytes) => Ok(bytes),
        FontSource::Url(url) => load_font_url(&url, None, cache).await,
    }
}

/// Load font bytes from URL with optional cache-busting version.
pub async fn load_font_url(
    url: &str,
    version: Option<&str>,
    cache: &mut FontCache,
) -> Result<Vec<u8>, WebFontLoadError> {
    if let Some(bytes) = cache.get_versioned(url, version) {
        return Ok(bytes.to_vec());
    }

    let bytes = fetch_font_bytes(url).await?;
    cache.insert_versioned(url, version, bytes.clone());
    Ok(bytes)
}

/// Errors returned by web font loader.
#[derive(Debug, thiserror::Error)]
pub enum WebFontLoadError {
    #[error("web font loading is only available on wasm32 targets")]
    UnsupportedPlatform,
    #[error("web_sys::window() is unavailable")]
    MissingWindow,
    #[error("failed to fetch font bytes: {0}")]
    Fetch(String),
    #[error("font request failed with HTTP status {0}")]
    HttpStatus(u16),
    #[error("failed to decode fetched font bytes: {0}")]
    Decode(String),
}

#[cfg(target_arch = "wasm32")]
async fn fetch_font_bytes(url: &str) -> Result<Vec<u8>, WebFontLoadError> {
    use wasm_bindgen::JsCast;

    let window = web_sys::window().ok_or(WebFontLoadError::MissingWindow)?;
    let response_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(url))
        .await
        .map_err(|e| WebFontLoadError::Fetch(format!("{e:?}")))?;
    let response = response_value
        .dyn_into::<web_sys::Response>()
        .map_err(|e| WebFontLoadError::Decode(format!("{e:?}")))?;

    if !response.ok() {
        return Err(WebFontLoadError::HttpStatus(response.status()));
    }

    let array_buffer = wasm_bindgen_futures::JsFuture::from(
        response
            .array_buffer()
            .map_err(|e| WebFontLoadError::Decode(format!("{e:?}")))?,
    )
    .await
    .map_err(|e| WebFontLoadError::Decode(format!("{e:?}")))?;

    Ok(js_sys::Uint8Array::new(&array_buffer).to_vec())
}

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_font_bytes(_url: &str) -> Result<Vec<u8>, WebFontLoadError> {
    Err(WebFontLoadError::UnsupportedPlatform)
}
