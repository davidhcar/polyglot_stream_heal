#!/usr/bin/env python3
"""Runnable Python (PyO3 Native Extension) Example for polyglot-stream-heal."""

import json
import polyglot_stream_heal as psh

print("=== 1. Stateful Streaming Transducer (Python PyO3) ===")
transducer = psh.IncrementalTransducer(max_table_cols=5, max_chart_bars=10)
stream_chunks = [
    "### Telemetry Stream\n```json\n",
    '{"service": "router", "active": tru, "p99_ms": 12.',
]
for chunk in stream_chunks:
    transducer.push_chunk(chunk)

active_seg = json.loads(transducer.snapshot_active_json())
print("Active Segment Modality:", active_seg["modality"])
print("Healed JSON Content    :", active_seg["content"])

print("\n=== 2. Context-Free Mermaid Dyck Node Healing ===")
raw_mermaid = "flowchart TD\n  subgraph Shard_A (Primary: US-East)\n    N1[Compute O(f(g(x)))] -- rpc --> N2[(Ledger (v2))]"
print(psh.heal_mermaid(raw_mermaid))
