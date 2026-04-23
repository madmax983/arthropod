use text_engine::{load_font_source, load_font_url, FontCache, FontSource};

#[test]
fn test_font_cache_returns_cached_bytes_for_same_url() {
    let mut cache = FontCache::default();
    cache.insert("https://x/font.ttf", vec![1, 2, 3]);
    assert_eq!(cache.get("https://x/font.ttf").unwrap(), &[1, 2, 3]);
}

#[test]
fn test_font_cache_uses_versioned_key() {
    let mut cache = FontCache::default();
    cache.insert_versioned("https://x/font.ttf", Some("42"), vec![9, 8, 7]);
    assert_eq!(
        cache
            .get_versioned("https://x/font.ttf", Some("42"))
            .unwrap(),
        &[9, 8, 7]
    );
    assert!(cache
        .get_versioned("https://x/font.ttf", Some("43"))
        .is_none());
}

#[test]
fn test_load_font_source_bytes_bypasses_network() {
    let mut cache = FontCache::default();
    let loaded = pollster::block_on(load_font_source(
        FontSource::Bytes(vec![5, 4, 3, 2]),
        &mut cache,
    ))
    .expect("bytes source should load");
    assert_eq!(loaded, vec![5, 4, 3, 2]);
}

#[test]
fn test_load_font_url_hits_cache_before_fetch() {
    let mut cache = FontCache::default();
    cache.insert_versioned("https://x/font.ttf", Some("v1"), vec![11, 22, 33]);
    let loaded = pollster::block_on(load_font_url("https://x/font.ttf", Some("v1"), &mut cache))
        .expect("cached url should load");
    assert_eq!(loaded, vec![11, 22, 33]);
}

#[test]
fn test_versioned_cache_key_with_empty_version() {
    // If version is provided but is empty, it should fall back to just URL
    assert_eq!(
        text_engine::versioned_cache_key("https://x/font.ttf", Some("")),
        "https://x/font.ttf"
    );
}

#[test]
fn test_fetch_font_bytes_unsupported_platform() {
    // In test environment, target_arch is NOT wasm32, so we expect unsupported
    let loaded = pollster::block_on(load_font_url(
        "https://x/font.ttf",
        None,
        &mut text_engine::FontCache::default(),
    ));
    assert!(matches!(
        loaded,
        Err(text_engine::WebFontLoadError::UnsupportedPlatform)
    ));
}
