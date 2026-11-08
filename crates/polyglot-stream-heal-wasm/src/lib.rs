use polyglot_stream_heal::{
    heal_html as core_heal_html, heal_json as core_heal_json, heal_mermaid as core_heal_mermaid,
    heal_stream as core_heal_stream, partition_visual_spec as core_partition,
    IncrementalTransducer, TransducerConfig,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmTransducer {
    inner: IncrementalTransducer,
}

#[wasm_bindgen]
impl WasmTransducer {
    #[wasm_bindgen(constructor)]
    pub fn new(
        max_table_cols: Option<usize>,
        max_chart_bars: Option<usize>,
        partition_visuals: Option<bool>,
    ) -> Self {
        let cfg = TransducerConfig {
            max_table_cols: max_table_cols.unwrap_or(5),
            max_chart_bars: max_chart_bars.unwrap_or(10),
            partition_visuals: partition_visuals.unwrap_or(true),
            repair_nested_escaped_json: true,
        };
        Self {
            inner: IncrementalTransducer::new(cfg),
        }
    }

    #[wasm_bindgen(js_name = pushChunk)]
    pub fn push_chunk(&mut self, chunk: &str) {
        self.inner.push_chunk(chunk);
    }

    #[wasm_bindgen(js_name = pushBytes)]
    pub fn push_bytes(&mut self, bytes: &[u8]) {
        self.inner.push_bytes(bytes);
    }

    #[wasm_bindgen(js_name = snapshotJson)]
    pub fn snapshot_json(&self) -> String {
        serde_json::to_string(&self.inner.snapshot()).unwrap_or_else(|_| "{}".to_string())
    }

    #[wasm_bindgen(js_name = snapshotActiveJson)]
    pub fn snapshot_active_json(&self) -> String {
        serde_json::to_string(&self.inner.snapshot_active_segment())
            .unwrap_or_else(|_| "null".to_string())
    }

    #[wasm_bindgen(js_name = snapshotMarkdown)]
    pub fn snapshot_markdown(&self) -> String {
        self.inner.snapshot().reconstructed_markdown
    }

    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = healJson)]
pub fn heal_json(raw: &str) -> String {
    core_heal_json(raw)
}

#[wasm_bindgen(js_name = healMermaid)]
pub fn heal_mermaid(raw: &str) -> String {
    core_heal_mermaid(raw)
}

#[wasm_bindgen(js_name = healHtml)]
pub fn heal_html(raw: &str) -> String {
    core_heal_html(raw)
}

#[wasm_bindgen(js_name = partitionVisualSpec)]
pub fn partition_visual_spec(
    raw_json: &str,
    max_table_cols: usize,
    max_chart_bars: usize,
) -> Result<String, JsValue> {
    core_partition(raw_json, max_table_cols, max_chart_bars).map_err(|e| JsValue::from_str(&e))
}

#[wasm_bindgen(js_name = healStreamJson)]
pub fn heal_stream_json(raw: &str) -> String {
    let snap = core_heal_stream(raw, TransducerConfig::default());
    serde_json::to_string(&snap).unwrap_or_else(|_| "{}".to_string())
}
