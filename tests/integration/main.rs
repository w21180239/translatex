//! Integration tests: engine -> provider -> HTTP -> typed error.
//!
//! Each test points a real provider adapter at a mock server, so the request
//! shape, response parsing and error mapping are exercised end to end without
//! touching a vendor endpoint.

use httpmock::prelude::*;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use translatex::core::config::Settings;
use translatex::core::creds::{Credential, CredentialStore};
use translatex::core::engine::Engine;
use translatex::core::types::TranslateRequest;
use translatex::providers::{libretranslate::LibreTranslate, Provider, Registry};

fn request(text: &str, target: &str) -> TranslateRequest {
    TranslateRequest {
        text: text.into(),
        source_lang: "en".into(),
        target_lang: target.into(),
        provider: None,
        fallback_chain: Vec::new(),
        preserve: Vec::new(),
        timeout_ms: None,
        idempotency_key: None,
        metadata: BTreeMap::new(),
    }
}

/// Retries off: these tests assert on the first attempt, not on backoff.
fn settings() -> Settings {
    Settings {
        retries: Some(0),
        ..Default::default()
    }
}

fn engine(registry: Registry, creds: CredentialStore, settings: Settings) -> Engine {
    // No cache dir: cooldown state stays in memory, so tests do not share it.
    Engine::new(Arc::new(registry), creds, None, settings)
}

#[tokio::test]
async fn libretranslate_round_trip() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST).path("/translate");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"translatedText":"你好世界","detectedSourceLanguage":"en"}"#);
    });

    let mut registry = Registry::new();
    registry.register(Arc::new(LibreTranslate::new(Some(&server.base_url()))));

    let out = engine(registry, CredentialStore::default(), settings())
        .translate(&request("Hello world", "zh"))
        .await
        .expect("translate");

    assert_eq!(out.response.text, "你好世界");
    assert_eq!(out.response.provider, "libretranslate");
    assert_eq!(out.response.target_lang, "zh");
    assert_eq!(out.response.detected_source, "en");
    assert_eq!(
        out.response.chars,
        Some(11),
        "source characters are counted"
    );
    assert_eq!(out.attempts.len(), 1);
    assert!(out.attempts[0].ok);
    assert_eq!(mock.hits(), 1);
}

#[tokio::test]
async fn upstream_failure_becomes_a_retryable_typed_error() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/translate");
        then.status(503).body("service unavailable");
    });

    let mut registry = Registry::new();
    registry.register(Arc::new(LibreTranslate::new(Some(&server.base_url()))));

    let mut req = request("hello", "zh");
    req.provider = Some("libretranslate".into());
    let err = engine(registry, CredentialStore::default(), settings())
        .translate(&req)
        .await
        .expect_err("503 must fail");

    assert_eq!(err.code(), "UPSTREAM_ERROR");
    assert!(err.retryable(), "5xx is retryable");
    assert!(err.to_string().contains("503"), "message names the status");
}

#[tokio::test]
async fn rate_limit_is_reported_with_the_backoff_action() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/translate");
        then.status(429).body("slow down");
    });

    let mut registry = Registry::new();
    registry.register(Arc::new(LibreTranslate::new(Some(&server.base_url()))));

    let mut req = request("hello", "zh");
    req.provider = Some("libretranslate".into());
    let err = engine(registry, CredentialStore::default(), settings())
        .translate(&req)
        .await
        .expect_err("429 must fail");

    assert_eq!(err.code(), "RATE_LIMITED");
    assert!(err.retryable());
    assert_eq!(err.suggested_action(), "backoff_and_retry");
}

#[tokio::test]
async fn malformed_body_is_a_bad_response_not_a_panic() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/translate");
        then.status(200)
            .header("content-type", "application/json")
            .body("<!doctype html><h1>blocked</h1>");
    });

    let mut registry = Registry::new();
    registry.register(Arc::new(LibreTranslate::new(Some(&server.base_url()))));

    let mut req = request("hello", "zh");
    req.provider = Some("libretranslate".into());
    let err = engine(registry, CredentialStore::default(), settings())
        .translate(&req)
        .await
        .expect_err("HTML is not a translation");

    assert_eq!(err.code(), "BAD_RESPONSE");
    assert!(!err.retryable());
}

#[tokio::test]
async fn engine_cuts_the_call_at_the_request_budget() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/translate");
        then.status(200)
            .delay(Duration::from_millis(700))
            .header("content-type", "application/json")
            .body(r#"{"translatedText":"晚到的译文"}"#);
    });

    let mut registry = Registry::new();
    registry.register(Arc::new(LibreTranslate::new(Some(&server.base_url()))));

    let mut req = request("hello", "zh");
    req.provider = Some("libretranslate".into());
    req.timeout_ms = Some(80);

    let started = Instant::now();
    let err = engine(registry, CredentialStore::default(), settings())
        .translate(&req)
        .await
        .expect_err("the budget must be enforced");
    let elapsed = started.elapsed();

    assert_eq!(err.code(), "TIMEOUT");
    assert!(
        elapsed < Duration::from_millis(500),
        "engine must stop at the 80 ms budget, took {elapsed:?}"
    );
}

#[tokio::test]
async fn probe_uses_the_liveness_endpoint() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET).path("/languages");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"[{"code":"en"},{"code":"zh"}]"#);
    });

    let provider = LibreTranslate::new(Some(&server.base_url()));
    let cred = Credential {
        provider: "libretranslate".into(),
        label: "default".into(),
        fields: BTreeMap::new(),
    };
    provider.probe(&cred).await.expect("healthy");
    assert_eq!(mock.hits(), 1);
}
