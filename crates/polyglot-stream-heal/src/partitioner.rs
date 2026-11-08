use serde_json::{json, Map, Value};

/// Cardinality-Bounded Layout Partitioner (Theorem 3(b)).
///
/// Enforces deterministic viewport invariants on structured visual specifications (`dynamic_visual` /
/// comparison tables / bar charts) so wide schemas never trigger horizontal overflow scrollbars:
/// - Wide tables with `columns.len() > max_table_cols` (default 5, keeping column 0 as the primary
///   anchor entity key) are partitioned into `ceil((C - 1) / (max_table_cols - 1))` vertical sub-tables,
///   each retaining column 0 plus at most `max_table_cols - 1` attribute columns.
/// - Dense bar/series charts with `bars.len() > max_chart_bars` (default 10) are partitioned into
///   contiguous sub-charts of size `<= max_chart_bars`.
/// - Preserves 100% of original cells and data points losslessly in O(|cells|) time.
pub fn partition_visual_spec(
    raw_json: &str,
    max_table_cols: usize,
    max_chart_bars: usize,
) -> Result<String, String> {
    let parsed: Value = serde_json::from_str(raw_json).map_err(|e| e.to_string())?;
    let partitioned = partition_value(parsed, max_table_cols.max(2), max_chart_bars.max(2));
    serde_json::to_string(&partitioned).map_err(|e| e.to_string())
}

pub fn partition_value(val: Value, max_cols: usize, max_bars: usize) -> Value {
    match val {
        Value::Object(mut map) => {
            // Case 1: Root container with `sections` or `cards` or `widgets` array
            for array_key in ["sections", "cards", "widgets", "items"] {
                if let Some(Value::Array(items)) = map.remove(array_key) {
                    let mut expanded = Vec::with_capacity(items.len());
                    for item in items {
                        expanded.extend(expand_component(item, max_cols, max_bars));
                    }
                    map.insert(array_key.to_string(), Value::Array(expanded));
                    return Value::Object(map);
                }
            }

            // Case 2: Standalone table or chart object -> if partitioned into >1 parts, wrap cleanly
            let parts = expand_component(Value::Object(map), max_cols, max_bars);
            if parts.len() == 1 {
                parts.into_iter().next().unwrap()
            } else {
                json!({
                    "layout": "adaptive_vertical_stack",
                    "partitioned": true,
                    "sections": parts
                })
            }
        }
        Value::Array(items) => {
            let mut expanded = Vec::with_capacity(items.len());
            for item in items {
                expanded.extend(expand_component(item, max_cols, max_bars));
            }
            Value::Array(expanded)
        }
        other => other,
    }
}

fn expand_component(item: Value, max_cols: usize, max_bars: usize) -> Vec<Value> {
    let Value::Object(map) = item else {
        return vec![item];
    };

    // Check wide table (`columns` / `headers` + `rows`)
    let col_key = if map.contains_key("columns") {
        Some("columns")
    } else if map.contains_key("headers") {
        Some("headers")
    } else {
        None
    };

    if let (Some(ck), Some(Value::Array(rows))) = (col_key, map.get("rows")) {
        if let Some(Value::Array(cols)) = map.get(ck) {
            if cols.len() > max_cols {
                return split_wide_table(&map, ck, cols, rows, max_cols);
            }
        }
    }

    // Check dense bar/series chart (`bars` or `data` array when kind/type indicates chart)
    for bar_key in ["bars", "series_points"] {
        if let Some(Value::Array(bars)) = map.get(bar_key) {
            if bars.len() > max_bars {
                return split_dense_bars(&map, bar_key, bars, max_bars);
            }
        }
    }

    vec![Value::Object(map)]
}

fn split_wide_table(
    base: &Map<String, Value>,
    col_key: &str,
    cols: &[Value],
    rows: &[Value],
    max_cols: usize,
) -> Vec<Value> {
    let stride = (max_cols - 1).max(1);
    let anchor_col = cols[0].clone();
    let attr_cols = &cols[1..];
    let total_parts = (attr_cols.len() + stride - 1) / stride;
    let base_title = base
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Comparison Table");

    let mut out = Vec::with_capacity(total_parts);
    for (part_idx, col_slice) in attr_cols.chunks(stride).enumerate() {
        let start_col_idx = 1 + part_idx * stride;
        let end_col_idx = start_col_idx + col_slice.len();

        let mut sub_cols = Vec::with_capacity(1 + col_slice.len());
        sub_cols.push(anchor_col.clone());
        sub_cols.extend_from_slice(col_slice);

        let mut sub_rows = Vec::with_capacity(rows.len());
        for row in rows {
            if let Value::Array(cells) = row {
                let mut sub_cells = Vec::with_capacity(1 + col_slice.len());
                sub_cells.push(cells.first().cloned().unwrap_or(Value::Null));
                for c_idx in start_col_idx..end_col_idx {
                    sub_cells.push(cells.get(c_idx).cloned().unwrap_or(Value::Null));
                }
                sub_rows.push(Value::Array(sub_cells));
            } else {
                sub_rows.push(row.clone());
            }
        }

        let mut sub_map = base.clone();
        sub_map.insert(
            "title".to_string(),
            Value::String(format!("{} (Part {}/{})", base_title, part_idx + 1, total_parts)),
        );
        sub_map.insert(col_key.to_string(), Value::Array(sub_cols));
        sub_map.insert("rows".to_string(), Value::Array(sub_rows));
        sub_map.insert("partitionIndex".to_string(), json!(part_idx + 1));
        sub_map.insert("partitionTotal".to_string(), json!(total_parts));
        out.push(Value::Object(sub_map));
    }
    out
}

fn split_dense_bars(
    base: &Map<String, Value>,
    bar_key: &str,
    bars: &[Value],
    max_bars: usize,
) -> Vec<Value> {
    let total_parts = (bars.len() + max_bars - 1) / max_bars;
    let base_title = base
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Comparison Chart");

    let mut out = Vec::with_capacity(total_parts);
    for (part_idx, chunk) in bars.chunks(max_bars).enumerate() {
        let mut sub_map = base.clone();
        sub_map.insert(
            "title".to_string(),
            Value::String(format!("{} (Segment {}/{})", base_title, part_idx + 1, total_parts)),
        );
        sub_map.insert(bar_key.to_string(), Value::Array(chunk.to_vec()));
        sub_map.insert("partitionIndex".to_string(), json!(part_idx + 1));
        sub_map.insert("partitionTotal".to_string(), json!(total_parts));
        out.push(Value::Object(sub_map));
    }
    out
}
