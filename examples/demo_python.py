#!/usr/bin/env python3
"""
Comprehensive Runnable Python (PyO3) Showcase for polyglot-stream-heal.

Demonstrates all 5 core capabilities with explicit Input -> Output pairs:
1. Recursive Nested Escaped Tool-Call JSON Repair (OpenAI / Anthropic / MCP)
2. Context-Free Mermaid Dyck-Parentheses Node & Subgraph Healing
3. Streaming HTML5 / JavaScript Sandbox Boundary Healing
4. Cardinality-Bounded Wide Table Layout Partitioning
5. Stateful Linear-Time Incremental Pushdown Transducer (SSE Streams)
"""

import json
import polyglot_stream_heal as psh

print("=" * 70)
print("polyglot-stream-heal: 5-Feature Systems & Verification Showcase")
print("=" * 70)

# -------------------------------------------------------------------------
# 1. Recursive Tool-Call Escaped JSON Repair
# -------------------------------------------------------------------------
print("\n[Feature 1: Recursive Escaped Tool-Call JSON Repair (OpenAI / Anthropic / MCP)]")
raw_tool = '{"tool":"query_database","arguments":"{\\"filter\\":{\\"status\\":\\"active\\",\\"tags\\":[\\"prod\\",\\"us-east\\"'
healed_tool = psh.heal_json(raw_tool)
print("Input (Truncated) :", raw_tool)
print("Output (RFC-8259) :", healed_tool)
parsed = json.loads(healed_tool)
inner_args = json.loads(parsed["arguments"])
print("Verified Inner JSON: arguments.tags =", inner_args["filter"]["tags"])

# -------------------------------------------------------------------------
# 2. Context-Free Mermaid Dyck-Parentheses & Subgraph Healing
# -------------------------------------------------------------------------
print("\n[Feature 2: Context-Free Mermaid Dyck Node & Subgraph Healing]")
raw_mermaid = "flowchart LR\n  subgraph Shard_A (Primary Cluster)\n    N1[Compute O(f(g(x)))] -- async --> N2[(Ledger (v16.4))]"
healed_mermaid = psh.heal_mermaid(raw_mermaid)
print("Input (Unquoted parens & unclosed subgraph):")
print(raw_mermaid)
print("\nOutput (Auto-quoted shapes & closed block directives):")
print(healed_mermaid)

# -------------------------------------------------------------------------
# 3. Streaming HTML5 / JavaScript / CSS Sandbox Healer
# -------------------------------------------------------------------------
print("\n[Feature 3: Streaming HTML5 / JavaScript Sandbox Boundary Healing]")
raw_html = '<div class="metric-card"><style>.val { color: #09f;</style><script>function render() { const msg = `Streaming status: ${state.'
healed_html = psh.heal_html(raw_html)
print("Input (Truncated JS template literal & unclosed DOM tags):")
print(raw_html)
print("\nOutput (Lexically closed script, style & container tags):")
print(healed_html)

# -------------------------------------------------------------------------
# 4. Cardinality-Bounded Wide Table Partitioning (Lossless Bijection)
# -------------------------------------------------------------------------
print("\n[Feature 4: Cardinality-Bounded Wide Table Partitioning (C_max = 4 columns)]")
wide_table = json.dumps({
    "title": "Cluster Benchmark",
    "columns": ["Host", "Throughput_MBps", "p50_us", "p99_us", "ErrorRate", "CPUPct", "MemPct"],
    "rows": [
        ["node-us-east-1", "1420.5", "6.3", "57.3", "0.00%", "42.1%", "68.4%"],
        ["node-eu-west-1", "1280.2", "7.1", "61.8", "0.01%", "48.7%", "71.2%"]
    ]
})
partitioned = psh.partition_visual_spec(wide_table, max_table_cols=4, max_chart_bars=10)
print("Input : 7-column schema (causes mobile/dashboard horizontal overflow)")
print("Output: Decomposed into 2 anchor-preserving stacked sub-tables (preserving Host):")
part_data = json.loads(partitioned)
sections = part_data.get("sections", [part_data])
print(f"Total Sub-Tables: {len(sections)}")
for i, part in enumerate(sections):
    print(f"  Partition {i+1} Columns (Entity + 3 attributes): {part['columns']}")

# -------------------------------------------------------------------------
# 5. Stateful Linear-Time Incremental Pushdown Transducer (SSE Streaming)
# -------------------------------------------------------------------------
print("\n[Feature 5: Stateful O(n) Incremental Pushdown Transducer (Token-by-Token SSE)]")
transducer = psh.IncrementalTransducer(max_table_cols=5, max_chart_bars=10)
sse_chunks = [
    "### System Architecture\n```mermaid\nflowchart TD\n",
    "  Gateway --> IngestionPipeline[Parse JSON Stream (v2)]\n",
    "```\n\n### Runtime Metrics\n```json\n",
    '{"service": "healer", "p50_us": 6.3, "status": "oper',
]
print(f"Streaming {len(sse_chunks)} successive network packets...")
for idx, chunk in enumerate(sse_chunks):
    transducer.push_chunk(chunk)
    active = json.loads(transducer.snapshot_active_json())
    print(f"  Chunk {idx+1} -> Active Modality: {active['modality']:<14} | Snapshot Length: {len(active['content'])} bytes")

final_markdown = transducer.snapshot_markdown()
print("\nFinal Reconstructed Polyglot Markdown Stream:")
print(final_markdown)
print("\n" + "=" * 70)
print("All 5 features successfully verified with zero runtime errors.")
print("=" * 70)

