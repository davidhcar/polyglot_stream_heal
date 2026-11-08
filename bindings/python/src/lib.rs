use ::polyglot_stream_heal::{
    heal_html as core_heal_html, heal_json as core_heal_json, heal_mermaid as core_heal_mermaid,
    heal_stream as core_heal_stream, partition_visual_spec as core_partition,
    IncrementalTransducer as CoreTransducer, TransducerConfig,
};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyclass(name = "IncrementalTransducer")]
pub struct PyIncrementalTransducer {
    inner: CoreTransducer,
}

#[pymethods]
impl PyIncrementalTransducer {
    #[new]
    #[pyo3(signature = (max_table_cols=5, max_chart_bars=10, partition_visuals=true, repair_nested_escaped_json=true))]
    pub fn new(
        max_table_cols: usize,
        max_chart_bars: usize,
        partition_visuals: bool,
        repair_nested_escaped_json: bool,
    ) -> Self {
        Self {
            inner: CoreTransducer::new(TransducerConfig {
                max_table_cols,
                max_chart_bars,
                partition_visuals,
                repair_nested_escaped_json,
            }),
        }
    }

    pub fn push_chunk(&mut self, chunk: &str) {
        self.inner.push_chunk(chunk);
    }

    pub fn push_bytes(&mut self, bytes: &[u8]) {
        self.inner.push_bytes(bytes);
    }

    pub fn snapshot_json(&self) -> String {
        serde_json::to_string(&self.inner.snapshot()).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn snapshot_active_json(&self) -> String {
        serde_json::to_string(&self.inner.snapshot_active_segment())
            .unwrap_or_else(|_| "null".to_string())
    }

    pub fn snapshot_markdown(&self) -> String {
        self.inner.snapshot().reconstructed_markdown
    }

    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[pyfunction]
pub fn heal_json(raw: &str) -> String {
    core_heal_json(raw)
}

#[pyfunction]
pub fn heal_mermaid(raw: &str) -> String {
    core_heal_mermaid(raw)
}

#[pyfunction]
pub fn heal_html(raw: &str) -> String {
    core_heal_html(raw)
}

#[pyfunction]
#[pyo3(signature = (raw_json, max_table_cols=5, max_chart_bars=10))]
pub fn partition_visual_spec(
    raw_json: &str,
    max_table_cols: usize,
    max_chart_bars: usize,
) -> PyResult<String> {
    core_partition(raw_json, max_table_cols, max_chart_bars).map_err(PyValueError::new_err)
}

#[pyfunction]
pub fn heal_stream_json(raw: &str) -> String {
    let snap = core_heal_stream(raw, TransducerConfig::default());
    serde_json::to_string(&snap).unwrap_or_else(|_| "{}".to_string())
}

#[pymodule]
fn polyglot_stream_heal(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyIncrementalTransducer>()?;
    m.add_function(wrap_pyfunction!(heal_json, m)?)?;
    m.add_function(wrap_pyfunction!(heal_mermaid, m)?)?;
    m.add_function(wrap_pyfunction!(heal_html, m)?)?;
    m.add_function(wrap_pyfunction!(partition_visual_spec, m)?)?;
    m.add_function(wrap_pyfunction!(heal_stream_json, m)?)?;
    Ok(())
}
