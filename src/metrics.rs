//! Well-known metrics for Ryzanstein observability.

/// Standard metric names
pub struct MetricNames;

impl MetricNames {
    // Inference metrics
    pub const INFERENCE_REQUESTS: &'static str = "ryzanstein.inference.requests";
    pub const INFERENCE_TOKENS: &'static str = "ryzanstein.inference.tokens";
    pub const INFERENCE_LATENCY_MS: &'static str = "ryzanstein.inference.latency_ms";
    pub const INFERENCE_ERRORS: &'static str = "ryzanstein.inference.errors";

    // Model metrics
    pub const MODEL_LOAD_TIME_MS: &'static str = "ryzanstein.model.load_time_ms";
    pub const MODEL_MEMORY_MB: &'static str = "ryzanstein.model.memory_mb";

    // KV Cache metrics
    pub const KV_CACHE_HIT_RATE: &'static str = "ryzanstein.kv_cache.hit_rate";
    pub const KV_CACHE_SIZE_MB: &'static str = "ryzanstein.kv_cache.size_mb";
    pub const KV_CACHE_EVICTIONS: &'static str = "ryzanstein.kv_cache.evictions";

    // Speculative decoding metrics
    pub const SPEC_ACCEPTANCE_RATE: &'static str = "ryzanstein.speculative.acceptance_rate";
    pub const SPEC_DRAFT_TOKENS: &'static str = "ryzanstein.speculative.draft_tokens";

    // Agent metrics
    pub const AGENT_EXECUTIONS: &'static str = "ryzanstein.agent.executions";
    pub const AGENT_LATENCY_MS: &'static str = "ryzanstein.agent.latency_ms";

    // System metrics
    pub const GPU_UTILIZATION: &'static str = "ryzanstein.system.gpu_utilization";
    pub const MEMORY_USAGE_MB: &'static str = "ryzanstein.system.memory_usage_mb";
    pub const THROUGHPUT_TPS: &'static str = "ryzanstein.system.throughput_tps";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_names_prefixed() {
        assert!(MetricNames::INFERENCE_REQUESTS.starts_with("ryzanstein."));
        assert!(MetricNames::GPU_UTILIZATION.starts_with("ryzanstein."));
    }

    #[test]
    fn test_metric_names_unique() {
        let names = vec![
            MetricNames::INFERENCE_REQUESTS,
            MetricNames::INFERENCE_TOKENS,
            MetricNames::INFERENCE_LATENCY_MS,
            MetricNames::MODEL_LOAD_TIME_MS,
            MetricNames::KV_CACHE_HIT_RATE,
            MetricNames::SPEC_ACCEPTANCE_RATE,
            MetricNames::AGENT_EXECUTIONS,
            MetricNames::GPU_UTILIZATION,
        ];
        let unique: std::collections::HashSet<_> = names.iter().collect();
        assert_eq!(names.len(), unique.len());
    }
}
