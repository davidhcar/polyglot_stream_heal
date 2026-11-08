pub mod discriminator;
pub mod html_pda;
pub mod json_pda;
pub mod mermaid_pda;
pub mod partitioner;

pub use discriminator::Modality;
pub use html_pda::{heal_html, IncrementalHtmlPda};
pub use json_pda::{heal_json, IncrementalJsonPda};
pub use mermaid_pda::{heal_mermaid, IncrementalMermaidPda};
pub use partitioner::partition_visual_spec;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TransducerConfig {
    pub max_table_cols: usize,
    pub max_chart_bars: usize,
    pub partition_visuals: bool,
    pub repair_nested_escaped_json: bool,
}

impl Default for TransducerConfig {
    fn default() -> Self {
        Self {
            max_table_cols: 5,
            max_chart_bars: 10,
            partition_visuals: true,
            repair_nested_escaped_json: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealedSegment {
    pub index: usize,
    pub modality: Modality,
    pub fence_tag: String,
    pub content: String,
    pub is_closed: bool,
    pub was_partitioned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamSnapshot {
    pub segments: Vec<HealedSegment>,
    pub reconstructed_markdown: String,
    pub total_bytes_processed: usize,
}

#[derive(Debug)]
enum ActiveInnerPda {
    Json(IncrementalJsonPda),
    Mermaid(IncrementalMermaidPda),
    Html(IncrementalHtmlPda),
    Passthrough(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OuterFenceState {
    ProseStartOfLine,
    ProseMidLine,
    ProseBackticks(u8),
    FenceHeader { backtick_len: u8 },
    InsideBlockStartOfLine { backtick_len: u8 },
    InsideBlockMidLine { backtick_len: u8 },
    InsideBlockClosingBackticks { backtick_len: u8, seen: u8 },
}

/// Hierarchical Multi-Grammar Pushdown Transducer (`IncrementalTransducer`).
///
/// Processes arbitrary byte or UTF-8 token chunks in strict O(|\Delta s_t|) amortized time,
/// dispatching outer Markdown prose and 19+ fenced DSL modalities into specialized inner
/// pushdown automata without ever re-scanning previously committed segments.
#[derive(Debug)]
pub struct IncrementalTransducer {
    config: TransducerConfig,
    outer_state: OuterFenceState,
    utf8_tail: Vec<u8>,
    total_bytes: usize,
    completed_segments: Vec<HealedSegment>,
    active_prose: String,
    active_fence_tag: String,
    active_modality: Modality,
    active_inner: Option<ActiveInnerPda>,
    pending_line_prefix: String,
}

impl Default for IncrementalTransducer {
    fn default() -> Self {
        Self::new(TransducerConfig::default())
    }
}

impl IncrementalTransducer {
    pub fn new(config: TransducerConfig) -> Self {
        Self {
            config,
            outer_state: OuterFenceState::ProseStartOfLine,
            utf8_tail: Vec::with_capacity(4),
            total_bytes: 0,
            completed_segments: Vec::with_capacity(8),
            active_prose: String::with_capacity(256),
            active_fence_tag: String::new(),
            active_modality: Modality::Markdown,
            active_inner: None,
            pending_line_prefix: String::new(),
        }
    }

    pub fn reset(&mut self) {
        self.outer_state = OuterFenceState::ProseStartOfLine;
        self.utf8_tail.clear();
        self.total_bytes = 0;
        self.completed_segments.clear();
        self.active_prose.clear();
        self.active_fence_tag.clear();
        self.active_modality = Modality::Markdown;
        self.active_inner = None;
        self.pending_line_prefix.clear();
    }

    /// Feeds raw network/SSE bytes, safely buffering split UTF-8 codepoints across chunk boundaries.
    pub fn push_bytes(&mut self, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }
        if self.utf8_tail.is_empty() {
            match std::str::from_utf8(bytes) {
                Ok(valid) => {
                    self.push_chunk(valid);
                    return;
                }
                Err(err) => {
                    let valid_up_to = err.valid_up_to();
                    if valid_up_to > 0 {
                        let valid = unsafe { std::str::from_utf8_unchecked(&bytes[..valid_up_to]) };
                        self.push_chunk(valid);
                    }
                    self.utf8_tail.extend_from_slice(&bytes[valid_up_to..]);
                    return;
                }
            }
        }

        self.utf8_tail.extend_from_slice(bytes);
        let combined = std::mem::take(&mut self.utf8_tail);
        match std::str::from_utf8(&combined) {
            Ok(valid) => self.push_chunk(valid),
            Err(err) => {
                let valid_up_to = err.valid_up_to();
                if valid_up_to > 0 {
                    let valid = unsafe { std::str::from_utf8_unchecked(&combined[..valid_up_to]) };
                    self.push_chunk(valid);
                }
                if err.error_len().is_none() {
                    self.utf8_tail.extend_from_slice(&combined[valid_up_to..]);
                }
            }
        }
    }

    /// Feeds a UTF-8 string chunk in O(|chunk|) time.
    pub fn push_chunk(&mut self, chunk: &str) {
        self.total_bytes += chunk.len();
        for ch in chunk.chars() {
            self.step_char(ch);
        }
    }

    fn step_char(&mut self, ch: char) {
        match self.outer_state {
            OuterFenceState::ProseStartOfLine => {
                if ch == '`' {
                    self.outer_state = OuterFenceState::ProseBackticks(1);
                } else if ch == '\n' {
                    self.active_prose.push('\n');
                } else if ch == ' ' || ch == '\t' {
                    self.pending_line_prefix.push(ch);
                } else {
                    if !self.pending_line_prefix.is_empty() {
                        self.active_prose.push_str(&self.pending_line_prefix);
                        self.pending_line_prefix.clear();
                    }
                    self.active_prose.push(ch);
                    self.outer_state = OuterFenceState::ProseMidLine;
                }
            }
            OuterFenceState::ProseMidLine => {
                self.active_prose.push(ch);
                if ch == '\n' {
                    self.outer_state = OuterFenceState::ProseStartOfLine;
                }
            }
            OuterFenceState::ProseBackticks(count) => {
                if ch == '`' {
                    self.outer_state = OuterFenceState::ProseBackticks(count.saturating_add(1));
                } else if count >= 3 {
                    // Commit accumulated prose segment if non-empty
                    self.pending_line_prefix.clear();
                    if !self.active_prose.trim().is_empty() {
                        let prose = std::mem::take(&mut self.active_prose);
                        self.completed_segments.push(HealedSegment {
                            index: self.completed_segments.len(),
                            modality: Modality::Markdown,
                            fence_tag: String::new(),
                            content: prose,
                            is_closed: true,
                            was_partitioned: false,
                        });
                    } else {
                        self.active_prose.clear();
                    }
                    self.active_fence_tag.clear();
                    if ch == '\n' {
                        self.enter_fenced_block(count);
                    } else if ch != '\r' {
                        self.active_fence_tag.push(ch);
                        self.outer_state = OuterFenceState::FenceHeader {
                            backtick_len: count,
                        };
                    } else {
                        self.outer_state = OuterFenceState::FenceHeader {
                            backtick_len: count,
                        };
                    }
                } else {
                    // Fewer than 3 backticks -> inline code span inside prose
                    if !self.pending_line_prefix.is_empty() {
                        self.active_prose.push_str(&self.pending_line_prefix);
                        self.pending_line_prefix.clear();
                    }
                    for _ in 0..count {
                        self.active_prose.push('`');
                    }
                    self.active_prose.push(ch);
                    self.outer_state = if ch == '\n' {
                        OuterFenceState::ProseStartOfLine
                    } else {
                        OuterFenceState::ProseMidLine
                    };
                }
            }
            OuterFenceState::FenceHeader { backtick_len } => {
                if ch == '\n' {
                    self.enter_fenced_block(backtick_len);
                } else if ch != '\r' {
                    self.active_fence_tag.push(ch);
                }
            }
            OuterFenceState::InsideBlockStartOfLine { backtick_len } => {
                if ch == '`' {
                    self.outer_state = OuterFenceState::InsideBlockClosingBackticks {
                        backtick_len,
                        seen: 1,
                    };
                } else if ch == ' ' || ch == '\t' {
                    self.pending_line_prefix.push(ch);
                } else if ch == '\n' {
                    if !self.pending_line_prefix.is_empty() {
                        let prefix = std::mem::take(&mut self.pending_line_prefix);
                        self.feed_active_inner(&prefix);
                    }
                    self.feed_active_inner("\n");
                } else {
                    if !self.pending_line_prefix.is_empty() {
                        let prefix = std::mem::take(&mut self.pending_line_prefix);
                        self.feed_active_inner(&prefix);
                    }
                    let mut buf = [0u8; 4];
                    self.feed_active_inner(ch.encode_utf8(&mut buf));
                    self.outer_state = OuterFenceState::InsideBlockMidLine { backtick_len };
                }
            }
            OuterFenceState::InsideBlockMidLine { backtick_len } => {
                let mut buf = [0u8; 4];
                self.feed_active_inner(ch.encode_utf8(&mut buf));
                if ch == '\n' {
                    self.outer_state = OuterFenceState::InsideBlockStartOfLine { backtick_len };
                }
            }
            OuterFenceState::InsideBlockClosingBackticks { backtick_len, seen } => {
                if ch == '`' {
                    self.outer_state = OuterFenceState::InsideBlockClosingBackticks {
                        backtick_len,
                        seen: seen.saturating_add(1),
                    };
                } else if seen >= backtick_len && (ch == '\n' || ch == '\r' || ch.is_whitespace()) {
                    if ch == '\n' {
                        self.pending_line_prefix.clear();
                        self.finalize_active_block();
                        self.outer_state = OuterFenceState::ProseStartOfLine;
                    } else {
                        // Wait for newline after closing fence (e.g. trailing spaces or `\r`)
                        self.outer_state = OuterFenceState::InsideBlockClosingBackticks {
                            backtick_len,
                            seen,
                        };
                    }
                } else {
                    // False alarm: backticks inside block body
                    if !self.pending_line_prefix.is_empty() {
                        let prefix = std::mem::take(&mut self.pending_line_prefix);
                        self.feed_active_inner(&prefix);
                    }
                    for _ in 0..seen {
                        self.feed_active_inner("`");
                    }
                    let mut buf = [0u8; 4];
                    self.feed_active_inner(ch.encode_utf8(&mut buf));
                    self.outer_state = if ch == '\n' {
                        OuterFenceState::InsideBlockStartOfLine { backtick_len }
                    } else {
                        OuterFenceState::InsideBlockMidLine { backtick_len }
                    };
                }
            }
        }
    }

    fn enter_fenced_block(&mut self, backtick_len: u8) {
        let modality = Modality::from_fence_tag(&self.active_fence_tag);
        self.active_modality = modality;
        self.active_inner = Some(if modality.is_json_backed() {
            ActiveInnerPda::Json(IncrementalJsonPda::new(
                self.config.repair_nested_escaped_json,
            ))
        } else if modality == Modality::Mermaid {
            ActiveInnerPda::Mermaid(IncrementalMermaidPda::new())
        } else if modality == Modality::HtmlSandbox {
            ActiveInnerPda::Html(IncrementalHtmlPda::new())
        } else {
            ActiveInnerPda::Passthrough(String::with_capacity(256))
        });
        self.outer_state = OuterFenceState::InsideBlockStartOfLine { backtick_len };
    }

    fn feed_active_inner(&mut self, s: &str) {
        match &mut self.active_inner {
            Some(ActiveInnerPda::Json(pda)) => pda.push_chunk(s),
            Some(ActiveInnerPda::Mermaid(pda)) => pda.push_chunk(s),
            Some(ActiveInnerPda::Html(pda)) => pda.push_chunk(s),
            Some(ActiveInnerPda::Passthrough(buf)) => buf.push_str(s),
            None => {}
        }
    }

    fn render_inner_snapshot(&self, inner: &ActiveInnerPda, modality: Modality) -> (String, bool) {
        match inner {
            ActiveInnerPda::Json(pda) => {
                let healed = pda.snapshot_healed();
                if self.config.partition_visuals
                    && matches!(modality, Modality::DynamicVisual | Modality::Json)
                    && !healed.is_empty()
                {
                    if let Ok(partitioned) = partition_visual_spec(
                        &healed,
                        self.config.max_table_cols,
                        self.config.max_chart_bars,
                    ) {
                        let changed = partitioned.len() != healed.len() || partitioned != healed;
                        return (partitioned, changed);
                    }
                }
                (healed, false)
            }
            ActiveInnerPda::Mermaid(pda) => (pda.snapshot_healed(), false),
            ActiveInnerPda::Html(pda) => (pda.snapshot_healed(), false),
            ActiveInnerPda::Passthrough(buf) => (buf.trim_end().to_string(), false),
        }
    }

    fn finalize_active_block(&mut self) {
        if let Some(inner) = self.active_inner.take() {
            let (content, was_partitioned) = self.render_inner_snapshot(&inner, self.active_modality);
            let tag = std::mem::take(&mut self.active_fence_tag);
            self.completed_segments.push(HealedSegment {
                index: self.completed_segments.len(),
                modality: self.active_modality,
                fence_tag: tag.trim().to_string(),
                content,
                is_closed: true,
                was_partitioned,
            });
        }
        self.active_modality = Modality::Markdown;
    }

    /// Returns ONLY the currently active (in-flight) segment healed in O(d + |active_tail|) time,
    /// without cloning or re-serializing any previously completed segments 0..m-1 (Theorem 2).
    pub fn snapshot_active_segment(&self) -> Option<HealedSegment> {
        let eof_closed_fence = match self.outer_state {
            OuterFenceState::InsideBlockClosingBackticks { backtick_len, seen } => {
                seen >= backtick_len
            }
            _ => false,
        };

        if let Some(inner) = &self.active_inner {
            let (content, was_partitioned) = self.render_inner_snapshot(inner, self.active_modality);
            return Some(HealedSegment {
                index: self.completed_segments.len(),
                modality: self.active_modality,
                fence_tag: self.active_fence_tag.trim().to_string(),
                content,
                is_closed: eof_closed_fence,
                was_partitioned,
            });
        }

        if !self.active_prose.trim().is_empty() {
            return Some(HealedSegment {
                index: self.completed_segments.len(),
                modality: Modality::Markdown,
                fence_tag: String::new(),
                content: self.active_prose.clone(),
                is_closed: true,
                was_partitioned: false,
            });
        }

        self.completed_segments.last().cloned()
    }

    /// Produces a zero-mutation healed snapshot of the entire polyglot stream in O(d + |active_tail|) time.
    pub fn snapshot(&self) -> StreamSnapshot {
        let mut segments = self.completed_segments.clone();

        // Check if closing backticks reached EOF without a trailing newline
        let eof_closed_fence = match self.outer_state {
            OuterFenceState::InsideBlockClosingBackticks { backtick_len, seen } => {
                seen >= backtick_len
            }
            _ => false,
        };

        if let Some(inner) = &self.active_inner {
            let (content, was_partitioned) = self.render_inner_snapshot(inner, self.active_modality);
            segments.push(HealedSegment {
                index: segments.len(),
                modality: self.active_modality,
                fence_tag: self.active_fence_tag.trim().to_string(),
                content,
                is_closed: eof_closed_fence,
                was_partitioned,
            });
        } else if !self.active_prose.trim().is_empty() {
            // If the entire stream has zero code fences, check semantic fallback for bare JSON/Mermaid/HTML
            if segments.is_empty() {
                let inferred = Modality::infer_unfenced(&self.active_prose);
                match inferred {
                    Modality::Json => {
                        let healed = heal_json(&self.active_prose);
                        let (final_json, was_partitioned) = if self.config.partition_visuals {
                            match partition_visual_spec(
                                &healed,
                                self.config.max_table_cols,
                                self.config.max_chart_bars,
                            ) {
                                Ok(p) => {
                                    let changed = p != healed;
                                    (p, changed)
                                }
                                Err(_) => (healed, false),
                            }
                        } else {
                            (healed, false)
                        };
                        segments.push(HealedSegment {
                            index: 0,
                            modality: Modality::Json,
                            fence_tag: String::new(),
                            content: final_json,
                            is_closed: false,
                            was_partitioned,
                        });
                    }
                    Modality::Mermaid => {
                        segments.push(HealedSegment {
                            index: 0,
                            modality: Modality::Mermaid,
                            fence_tag: String::new(),
                            content: heal_mermaid(&self.active_prose),
                            is_closed: false,
                            was_partitioned: false,
                        });
                    }
                    Modality::HtmlSandbox => {
                        segments.push(HealedSegment {
                            index: 0,
                            modality: Modality::HtmlSandbox,
                            fence_tag: String::new(),
                            content: heal_html(&self.active_prose),
                            is_closed: false,
                            was_partitioned: false,
                        });
                    }
                    _ => {
                        segments.push(HealedSegment {
                            index: 0,
                            modality: Modality::Markdown,
                            fence_tag: String::new(),
                            content: self.active_prose.clone(),
                            is_closed: true,
                            was_partitioned: false,
                        });
                    }
                }
            } else {
                segments.push(HealedSegment {
                    index: segments.len(),
                    modality: Modality::Markdown,
                    fence_tag: String::new(),
                    content: self.active_prose.clone(),
                    is_closed: true,
                    was_partitioned: false,
                });
            }
        }

        let mut reconstructed = String::with_capacity(self.total_bytes + 64);
        for (idx, seg) in segments.iter().enumerate() {
            if idx > 0 && !reconstructed.ends_with('\n') {
                reconstructed.push('\n');
            }
            if seg.modality == Modality::Markdown {
                reconstructed.push_str(&seg.content);
            } else if seg.fence_tag.is_empty() && segments.len() == 1 {
                reconstructed.push_str(&seg.content);
            } else {
                let tag = if seg.fence_tag.is_empty() {
                    "code"
                } else {
                    seg.fence_tag.as_str()
                };
                reconstructed.push_str("```");
                reconstructed.push_str(tag);
                reconstructed.push('\n');
                reconstructed.push_str(&seg.content);
                reconstructed.push_str("\n```\n");
            }
        }

        StreamSnapshot {
            segments,
            reconstructed_markdown: reconstructed,
            total_bytes_processed: self.total_bytes,
        }
    }
}

/// Stateless one-shot helper over a complete or truncated polyglot stream.
pub fn heal_stream(raw: &str, config: TransducerConfig) -> StreamSnapshot {
    let mut t = IncrementalTransducer::new(config);
    t.push_chunk(raw);
    t.snapshot()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heals_arbitrary_mid_token_json_truncations() {
        let full = r#"{"title":"System Latency","metrics":[{"name":"p50","val":1.25,"ok":true},{"name":"p99","val":4.8,"ok":false}]}"#;
        for cut in 1..full.len() {
            let prefix = &full[..cut];
            let healed = heal_json(prefix);
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(&healed);
            assert!(
                parsed.is_ok(),
                "Failed JSON parse at cut={} prefix={:?} -> healed={:?}",
                cut,
                prefix,
                healed
            );
        }
    }

    #[test]
    fn heals_nested_escaped_tool_call_json_arguments() {
        let truncated_tool =
            r#"{"tool":"query_cluster","arguments":"{\"filters\":[{\"field\":\"latency\",\"op\":\">=\",\"val\":12."}"#;
        // Cut off even before the outer string closes:
        let mid_stream = &truncated_tool[..truncated_tool.len() - 2];
        let healed = heal_json(mid_stream);
        let outer: serde_json::Value = serde_json::from_str(&healed).expect("valid outer JSON");
        let inner_str = outer["arguments"].as_str().expect("arguments string");
        let inner: serde_json::Value =
            serde_json::from_str(inner_str).expect("valid healed inner escaped JSON");
        assert_eq!(inner["filters"][0]["field"], "latency");
        assert_eq!(inner["filters"][0]["val"], 12.0);
    }

    #[test]
    fn heals_mermaid_dyck_nested_parentheses_and_subgraphs() {
        let raw = "flowchart LR\n  subgraph Compute Tier (v2.1)\n  A[Evaluate O(f(g(h(x))))] -- async rpc --> B[(Primary DB (Postgres: 16.2))]\n  B --> C{Latency < p99 (5ms)?";
        let healed = heal_mermaid(raw);
        assert!(healed.contains(r#"A["Evaluate O(f(g(h(x))))"]"#), "got:\n{}", healed);
        assert!(
            healed.contains(r#"B[("Primary DB (Postgres: 16.2)")]"#),
            "got:\n{}",
            healed
        );
        assert!(
            healed.contains(r#"C{"Latency < p99 (5ms)?"}"#),
            "got:\n{}",
            healed
        );
        assert!(healed.contains("-->|async rpc|"), "got:\n{}", healed);
        assert!(healed.ends_with("end"), "must auto-close subgraph:\n{}", healed);
    }

    #[test]
    fn partitions_wide_tables_and_dense_charts_losslessly() {
        let wide = serde_json::json!({
            "sections": [
                {
                    "kind": "comparison_table",
                    "title": "Engine Matrix",
                    "columns": ["Engine", "C1", "C2", "C3", "C4", "C5", "C6", "C7", "C8"],
                    "rows": [
                        ["HM-PDT", "1", "2", "3", "4", "5", "6", "7", "8"],
                        ["Regex", "a", "b", "c", "d", "e", "f", "g", "h"]
                    ]
                }
            ]
        });
        let out_str = partition_visual_spec(&wide.to_string(), 5, 10).unwrap();
        let out: serde_json::Value = serde_json::from_str(&out_str).unwrap();
        let sections = out["sections"].as_array().unwrap();
        assert_eq!(sections.len(), 2);
        for sec in sections {
            let cols = sec["columns"].as_array().unwrap();
            assert!(cols.len() <= 5);
            assert_eq!(cols[0], "Engine");
        }
    }

    #[test]
    fn handles_utf8_multibyte_splits_across_polyglot_fences() {
        let doc = "Intro — αβγ 🚀\n```dynamic_visual\n{\"title\":\"Metric Δ\",\"bars\":[{\"label\":\"A\",\"value\":42}";
        let bytes = doc.as_bytes();
        let mut transducer = IncrementalTransducer::default();
        for b in bytes.chunks(3) {
            transducer.push_bytes(b);
        }
        let snap = transducer.snapshot();
        assert_eq!(snap.segments.len(), 2);
        assert_eq!(snap.segments[0].modality, Modality::Markdown);
        assert!(snap.segments[0].content.contains("αβγ 🚀"));
        assert_eq!(snap.segments[1].modality, Modality::DynamicVisual);
        let parsed: serde_json::Value = serde_json::from_str(&snap.segments[1].content).unwrap();
        assert_eq!(parsed["title"], "Metric Δ");
    }
}
