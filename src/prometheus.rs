//! Prometheus text-format /metrics HTTP endpoint.

use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};

use crate::MetricsCollector;

/// Render all current metric values in Prometheus text exposition format.
pub fn format_metrics(metrics: &MetricsCollector) -> String {
    let mut out = String::with_capacity(1024);

    // Sanitize metric names: replace dots/hyphens with underscores.
    let sanitize = |s: &str| s.replace(['.', '-'], "_");

    // Counters
    {
        let counters = metrics.counters.lock().unwrap();
        let mut names: Vec<&String> = counters.keys().collect();
        names.sort();
        for name in names {
            let value = counters[name];
            let pname = sanitize(name);
            out.push_str(&format!("# HELP {pname} {name}\n"));
            out.push_str(&format!("# TYPE {pname} counter\n"));
            out.push_str(&format!("{pname} {value}\n"));
        }
    }

    // Gauges
    {
        let gauges = metrics.gauges.lock().unwrap();
        let mut names: Vec<&String> = gauges.keys().collect();
        names.sort();
        for name in names {
            let value = gauges[name];
            let pname = sanitize(name);
            out.push_str(&format!("# HELP {pname} {name}\n"));
            out.push_str(&format!("# TYPE {pname} gauge\n"));
            out.push_str(&format!("{pname} {value}\n"));
        }
    }

    // Histograms — expose p50, p99, sum, count as separate gauge/counter lines.
    {
        let histograms = metrics.histograms.lock().unwrap();
        let mut names: Vec<&String> = histograms.keys().collect();
        names.sort();
        for name in names {
            let values = &histograms[name];
            if values.is_empty() {
                continue;
            }
            let pname = sanitize(name);
            let count = values.len();
            let sum: f64 = values.iter().sum();
            let mut sorted = values.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let p50 = percentile_sorted(&sorted, 50.0);
            let p95 = percentile_sorted(&sorted, 95.0);
            let p99 = percentile_sorted(&sorted, 99.0);

            out.push_str(&format!("# HELP {pname} {name}\n"));
            out.push_str(&format!("# TYPE {pname} histogram\n"));
            out.push_str(&format!("{pname}_count {count}\n"));
            out.push_str(&format!("{pname}_sum {sum}\n"));
            out.push_str(&format!("{pname}_p50 {p50}\n"));
            out.push_str(&format!("{pname}_p95 {p95}\n"));
            out.push_str(&format!("{pname}_p99 {p99}\n"));
        }
    }

    out
}

fn percentile_sorted(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let n = sorted.len();
    if n == 1 {
        return sorted[0];
    }
    let rank = (p / 100.0) * (n - 1) as f64;
    let lo = rank.floor() as usize;
    let hi = rank.ceil() as usize;
    let frac = rank - lo as f64;
    sorted[lo] + frac * (sorted[hi] - sorted[lo])
}

/// Spawn a Tokio task that serves GET /metrics on `addr`.
/// The task runs until `.abort()` is called on the returned JoinHandle.
pub fn serve(addr: SocketAddr, metrics: Arc<MetricsCollector>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let make_svc = make_service_fn(move |_conn| {
            let metrics = Arc::clone(&metrics);
            async move {
                Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                    let metrics = Arc::clone(&metrics);
                    async move { Ok::<Response<Body>, Infallible>(handle(req, &metrics)) }
                }))
            }
        });

        if let Ok(server) = Server::try_bind(&addr) {
            let _ = server.serve(make_svc).await;
        }
    })
}

fn handle(req: Request<Body>, metrics: &MetricsCollector) -> Response<Body> {
    if req.uri().path() == "/metrics" {
        let body = format_metrics(metrics);
        Response::builder()
            .status(200)
            .header("Content-Type", "text/plain; version=0.0.4; charset=utf-8")
            .body(Body::from(body))
            .unwrap()
    } else {
        Response::builder()
            .status(404)
            .body(Body::from("not found"))
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MetricsCollector;

    #[test]
    fn test_format_counters() {
        let m = MetricsCollector::new();
        m.increment_by("ryzanstein_requests", 42);
        let out = format_metrics(&m);
        assert!(out.contains("ryzanstein_requests 42"), "got:\n{out}");
        assert!(out.contains("# TYPE ryzanstein_requests counter"));
    }

    #[test]
    fn test_format_gauges() {
        let m = MetricsCollector::new();
        m.set_gauge("ryzanstein_active", 7.0);
        let out = format_metrics(&m);
        assert!(out.contains("ryzanstein_active 7"), "got:\n{out}");
        assert!(out.contains("# TYPE ryzanstein_active gauge"));
    }

    #[test]
    fn test_format_histograms() {
        let m = MetricsCollector::new();
        for v in [1.0, 2.0, 3.0, 4.0, 5.0] {
            m.record_histogram("latency_ms", v);
        }
        let out = format_metrics(&m);
        assert!(out.contains("latency_ms_count 5"), "got:\n{out}");
        assert!(out.contains("latency_ms_sum 15"), "got:\n{out}");
    }

    #[test]
    fn test_dot_sanitization() {
        let m = MetricsCollector::new();
        m.increment_by("ryzanstein.inference.requests", 1);
        let out = format_metrics(&m);
        assert!(
            out.contains("ryzanstein_inference_requests 1"),
            "got:\n{out}"
        );
    }
}
