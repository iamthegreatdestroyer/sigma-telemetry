use sigma_telemetry::{config::TelemetryConfig, SigmaTelemetry, SpanOperation};
use std::time::Duration;

fn make_telemetry() -> SigmaTelemetry {
    SigmaTelemetry::new(TelemetryConfig::default())
}

#[test]
fn test_full_span_lifecycle() {
    let t = make_telemetry();

    let mut span = t.start_span("test_op", SpanOperation::Inference);
    span.set_attribute("model", "bitnet");

    t.metrics().increment("test.requests");
    t.metrics().record_histogram("test.latency", 42.0);

    span.set_ok();

    let snap = t.snapshot();
    assert_eq!(snap.span_count, 1, "expected 1 completed span");
    assert!(
        t.metrics().get_counter("test.requests") >= 1,
        "counter not recorded"
    );
    let stats = t.metrics().get_histogram_stats("test.latency").unwrap();
    assert_eq!(stats.count, 1);
    assert!((stats.sum - 42.0).abs() < 1e-9);
}

#[test]
fn test_window_stats_integration() {
    let t = make_telemetry();
    for v in [10.0, 20.0, 30.0, 40.0, 50.0] {
        t.metrics().record_histogram("test.window", v);
    }
    let ws = t
        .window_stats("test.window", Duration::from_secs(60))
        .expect("window stats should be present");
    assert!((ws.sum - 150.0).abs() < 1e-9, "sum={}", ws.sum);
    assert!((ws.mean - 30.0).abs() < 1e-9, "mean={}", ws.mean);
    assert!(ws.p99 >= 49.0, "p99={}", ws.p99);
    assert!(ws.rate > 0.0, "rate={}", ws.rate);
}

#[test]
fn test_multi_span_error_tracking() {
    let t = make_telemetry();

    t.start_span("ok_op", SpanOperation::Inference).set_ok();
    t.start_span("err_op", SpanOperation::TokenGeneration)
        .set_error("decode failure");

    assert_eq!(t.metrics().get_counter("spans.total"), 2);
    assert_eq!(t.metrics().get_counter("spans.errors"), 1);
}

#[tokio::test]
async fn test_prometheus_endpoint_format() {
    let t = std::sync::Arc::new(make_telemetry());

    t.metrics().increment_by("test_requests", 42);
    t.metrics().set_gauge("test_active", 7.0);

    let addr: std::net::SocketAddr = "127.0.0.1:19090".parse().unwrap();
    let handle = t.serve_metrics(addr);

    // Give the server a moment to bind
    tokio::time::sleep(Duration::from_millis(100)).await;

    let resp = reqwest::get("http://127.0.0.1:19090/metrics")
        .await
        .expect("GET /metrics failed");

    assert_eq!(resp.status().as_u16(), 200);
    let body = resp.text().await.unwrap();

    assert!(
        body.contains("test_requests 42"),
        "missing test_requests 42 in:\n{body}"
    );
    assert!(
        body.contains("test_active 7"),
        "missing test_active 7 in:\n{body}"
    );
    assert!(body.contains("# TYPE"), "missing # TYPE lines in:\n{body}");

    handle.abort();
}

#[tokio::test]
async fn test_prometheus_404_on_other_paths() {
    let t = make_telemetry();
    let addr: std::net::SocketAddr = "127.0.0.1:19091".parse().unwrap();
    let handle = t.serve_metrics(addr);

    tokio::time::sleep(Duration::from_millis(100)).await;

    let resp = reqwest::get("http://127.0.0.1:19091/health")
        .await
        .expect("GET /health failed");

    assert_eq!(resp.status().as_u16(), 404);

    handle.abort();
}
