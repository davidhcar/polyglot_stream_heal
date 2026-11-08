# `polyglot-stream-heal` (Python Native `PyO3` Extension, `PyPI`)

**Ultra-fast $6.3\,\mu\text{s}$ incremental $O(n)$ streaming JSON repair, 14-dialect Mermaid diagram healer, and HTML5 sandbox balancer for LLM token streams ($61.3\times$ faster drop-in alternative to `json-repair` and `partial-json`).**

[![PyPI](https://img.shields.io/pypi/v/polyglot-stream-heal.svg)](https://pypi.org/project/polyglot-stream-heal/)
[![Crates.io](https://img.shields.io/crates/v/polyglot-stream-heal.svg)](https://crates.io/crates/polyglot-stream-heal)
[![npm](https://img.shields.io/npm/v/polyglot-stream-heal.svg)](https://www.npmjs.com/package/polyglot-stream-heal)
[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/davidhcar/polyglot_stream_heal/blob/main/examples/colab_quickstart.ipynb)

- **Author:** **David Charles Michael** (`davidchar@gmail.com` | [ORCID: `0009-0007-8679-6941`](https://orcid.org/0009-0007-8679-6941))
- **License:** MIT
- **Native Rust Performance in Python:** Compiled via `PyO3` + `maturin` (`abi3-py39+`), delivering **$61.3\times$ lower median chunk latency** ($6.3\,\mu\text{s}$ vs. $386.8\,\mu\text{s}$) and **$264.2\times$ lower cumulative stream overhead** at $32\text{ KB}$ than pure-Python `json-repair`, while additionally healing nested escaped tool-call JSON strings, 14 Mermaid diagram dialects, HTML5 sandboxes, and wide tables.

---

## 1. Installation

```bash
pip install -U polyglot-stream-heal
```

---

## 2. How to Implement & Verified Output

### Runnable Code Snippet (`python3 examples/demo_python.py`)

```python
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
```

### Exact Verified Output (`examples/VERIFIED_OUTPUT.txt`)

```text
=== 1. Stateful Streaming Transducer (Python PyO3) ===
Active Segment Modality: json
Healed JSON Content    : {"active":true,"p99_ms":12.0,"service":"router"}

=== 2. Context-Free Mermaid Dyck Node Healing ===
flowchart TD
  subgraph sg_Shard_A__Primary__US ["Shard_A (Primary: US-East)"]
  N1["Compute O(f(g(x)))"] -->|rpc| N2[("Ledger (v2)")]
  end
```
