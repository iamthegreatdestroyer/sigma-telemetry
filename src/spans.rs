//! Well-known span definitions for Ryzanstein operations.

use crate::SpanOperation;

/// Pre-defined span templates for common Ryzanstein operations
pub struct SpanTemplates;

impl SpanTemplates {
    /// Inference request span with model and token attributes
    pub fn inference(model: &str, max_tokens: usize) -> (SpanOperation, Vec<(&'static str, String)>) {
        (
            SpanOperation::Inference,
            vec![
                ("model.name", model.to_string()),
                ("model.max_tokens", max_tokens.to_string()),
            ],
        )
    }

    /// Model loading span
    pub fn model_load(model: &str, size_mb: f64) -> (SpanOperation, Vec<(&'static str, String)>) {
        (
            SpanOperation::ModelLoad,
            vec![
                ("model.name", model.to_string()),
                ("model.size_mb", format!("{:.1}", size_mb)),
            ],
        )
    }

    /// KV cache operation span
    pub fn kv_cache(operation: &str, layer: usize) -> (SpanOperation, Vec<(&'static str, String)>) {
        (
            SpanOperation::KvCacheOp,
            vec![
                ("kv.operation", operation.to_string()),
                ("kv.layer", layer.to_string()),
            ],
        )
    }

    /// Speculative decoding draft span
    pub fn speculative_draft(draft_tokens: usize) -> (SpanOperation, Vec<(&'static str, String)>) {
        (
            SpanOperation::SpeculativeDraft,
            vec![("speculative.draft_tokens", draft_tokens.to_string())],
        )
    }

    /// Speculative decoding verification span
    pub fn speculative_verify(accepted: usize, total: usize) -> (SpanOperation, Vec<(&'static str, String)>) {
        (
            SpanOperation::SpeculativeVerify,
            vec![
                ("speculative.accepted", accepted.to_string()),
                ("speculative.total", total.to_string()),
                ("speculative.acceptance_rate", format!("{:.2}", accepted as f64 / total as f64)),
            ],
        )
    }

    /// Agent execution span
    pub fn agent_execute(agent_id: &str, capability: &str) -> (SpanOperation, Vec<(&'static str, String)>) {
        (
            SpanOperation::AgentExecute,
            vec![
                ("agent.id", agent_id.to_string()),
                ("agent.capability", capability.to_string()),
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_template() {
        let (op, attrs) = SpanTemplates::inference("bitnet-3b", 1024);
        assert_eq!(op, SpanOperation::Inference);
        assert_eq!(attrs.len(), 2);
        assert_eq!(attrs[0].1, "bitnet-3b");
    }

    #[test]
    fn test_model_load_template() {
        let (op, attrs) = SpanTemplates::model_load("mamba-2.8b", 5600.0);
        assert_eq!(op, SpanOperation::ModelLoad);
        assert_eq!(attrs[1].1, "5600.0");
    }

    #[test]
    fn test_speculative_verify_acceptance() {
        let (_, attrs) = SpanTemplates::speculative_verify(8, 10);
        assert_eq!(attrs[2].1, "0.80");
    }

    #[test]
    fn test_agent_execute_template() {
        let (op, attrs) = SpanTemplates::agent_execute("agent-001", "code_review");
        assert_eq!(op, SpanOperation::AgentExecute);
        assert_eq!(attrs[0].1, "agent-001");
    }
}
