use polyglot_stream_heal::{
    heal_html, heal_json, heal_mermaid, partition_visual_spec, IncrementalTransducer,
    TransducerConfig,
};

fn main() {
    println!("=== 1. Incremental Polyglot Stream Healing (Rust) ===");
    let mut transducer = IncrementalTransducer::new(TransducerConfig::default());
    let chunks = [
        "Live Architecture Overview:\n```mermaid\nflowchart LR\n",
        "  subgraph Compute Tier (v2.4)\n",
        "    A[Evaluate O(f(g(h(x))))] -- async batch --> B[(Primary DB (Postgres: 16.4))]",
    ];
    for chunk in chunks {
        transducer.push_chunk(chunk);
    }
    let snap = transducer.snapshot();
    println!("{}", snap.reconstructed_markdown);

    println!("=== 2. Truncated Escaped Tool-Call JSON Repair ===");
    let truncated_tool =
        r#"{"tool":"query_metrics","arguments":"{\"service\":\"gateway\",\"quantiles\":[0.5,0.99"#;
    let healed_tool = heal_json(truncated_tool);
    println!("Input : {}", truncated_tool);
    println!("Healed: {}\n", healed_tool);

    println!("=== 3. Context-Free Mermaid Dyck Parentheses & Subgraph Healing ===");
    let raw_mermaid = "flowchart TD\n  subgraph Shard_A (Primary: US-East)\n    N1[Compute O(f(g(x)))] --> N2[(Ledger (v2))]";
    println!("Healed Mermaid:\n{}\n", heal_mermaid(raw_mermaid));

    println!("=== 4. Streaming HTML5 / JavaScript Sandbox Boundary Healing ===");
    let raw_html = "<div class=\"widget\"><script>function calc(x) { const s = \"streaming_val";
    println!("Healed HTML5:\n{}\n", heal_html(raw_html));

    println!("=== 5. Cardinality-Bounded Wide Table Partitioning (max_cols = 4) ===");
    let wide_table = r#"{"title":"Engine Matrix","columns":["System","TrackA","TrackB","TrackC","Overall","p50_us","p99_us"],"rows":[["HM-PDT","100%","100%","100%","100%","6.4","63.7"]]}"#;
    let partitioned = partition_visual_spec(wide_table, 4, 10).unwrap();
    println!("{}\n", partitioned);
}
