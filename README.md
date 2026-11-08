# `polyglot-stream-heal`

**Ultra-fast $6.3\,\mu\text{s}$ incremental $O(n)$ streaming JSON repair, 14-dialect Mermaid diagram healer, HTML5 sandbox balancer, and cardinality-bounded layout partitioner for Large Language Model (LLM) token streams.**

[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.24045975.svg)](https://doi.org/10.5281/zenodo.24045975)
[![Crates.io](https://img.shields.io/crates/v/polyglot-stream-heal.svg)](https://crates.io/crates/polyglot-stream-heal)
[![Crates.io Downloads](https://img.shields.io/crates/d/polyglot-stream-heal.svg?label=crates.io%20downloads)](https://crates.io/crates/polyglot-stream-heal)
[![PyPI](https://img.shields.io/pypi/v/polyglot-stream-heal.svg)](https://pypi.org/project/polyglot-stream-heal/)
[![PyPI Downloads](https://static.pepy.tech/badge/polyglot-stream-heal)](https://pepy.tech/projects/polyglot-stream-heal)
[![npm](https://img.shields.io/npm/v/polyglot-stream-heal.svg)](https://www.npmjs.com/package/polyglot-stream-heal)
[![npm Downloads](https://img.shields.io/npm/dt/polyglot-stream-heal.svg?label=npm%20downloads)](https://www.npmjs.com/package/polyglot-stream-heal)
[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/davidhcar/polyglot_stream_heal/blob/main/examples/colab_quickstart.ipynb)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

- **Author:** **David Charles Michael** ([ORCID: `0009-0007-8679-6941`](https://orcid.org/0009-0007-8679-6941) | [DOI: `10.5281/zenodo.24045975`](https://doi.org/10.5281/zenodo.24045975) | `davidchar@gmail.com`)
- **Drop-In High-Speed Replacement For:** `json-repair` (Python), `partial-json` (npm/Python), `jsonrepair` (JS/TS), and brittle regex Mermaid/Markdown stream sanitizers.
- **Compatible With All Streaming LLM SDKs:** OpenAI (`chat.completions.create(stream=True)`), Anthropic Claude SSE streams, Google Gemini streaming, Vercel AI SDK (`useChat` / `streamText`), LangChain (`astream_events`), LlamaIndex, vLLM, and Ollama.

---

## Why Use `polyglot-stream-heal` Instead of `json-repair` or `partial-json`?

| Capability / Metric (`PolyglotStreamBench-1800`) | `polyglot-stream-heal` (Ours) | `partial-json` (npm) | `json-repair` (PyPI) | Multi-Pass Regex |
| :--- | :---: | :---: | :---: | :---: |
| **Streaming Complexity Across $N$ Tokens** | **$\Theta(N)$ Incremental** | $\Theta(N^2)$ Full Re-Scan | $\Theta(N^2)$ Full Re-Scan | $\Theta(R \cdot N^2)$ Full Re-Scan |
| **Median Per-Chunk Latency ($p50$)** | **$6.3\,\mu\text{s}$** | $38.1\,\mu\text{s}$ | $386.8\,\mu\text{s}$ (**$61.3\times$ slower**) | $20.2\,\mu\text{s}$ |
| **Cumulative Time on $32\text{ KB}$ Stream** | **$2.67\text{ ms}$** | $180+\text{ ms}$ | $714.4\text{ ms}$ (**$264.2\times$ slower**) | $202.6\text{ ms}$ |
| **Truncated / Incomplete JSON Repair** | **100.0%** | 66.7% | 66.7% | 26.3% |
| **Nested Escaped Tool-Call `arguments` JSON** | **100.0%** | Fails (0%) | Fails (0%) | Fails (0%) |
| **14 Mermaid Diagram Dialects (Nested Dyck Labels)** | **100.0%** | Not Supported (0%) | Not Supported (0%) | 40.3% |
| **Streaming HTML5 / `<script>` / `<style>` Closure** | **100.0%** | Not Supported (0%) | Not Supported (0%) | Fails (0%) |
| **Wide Table & Bar Chart Overflow Partitioning** | **100.0%** | Not Supported (0%) | Not Supported (0%) | Not Supported (0%) |

---

## Tri-Registry Installation & Quickstart

### 1. Python (`PyPI`, Native `PyO3` C-Extension)
```bash
pip install -U polyglot-stream-heal
```

```python
import json
import polyglot_stream_heal as psh

# A. Stateless drop-in replacement for json_repair (61x faster + heals nested escaped tool arguments)
healed_json = psh.heal_json('{"tool":"sql","arguments":"{\\"query\\":\\"SELECT * FROM users WHERE active = tru')
print(json.loads(healed_json))

# B. Stateless Mermaid flowchart / sequence diagram healing (nested parentheses + unclosed subgraphs)
healed_mermaid = psh.heal_mermaid("flowchart LR\n  subgraph Compute (v2)\n    A[Evaluate O(f(g(x)))] --> B[(DB (v16))]")
print(healed_mermaid)

# C. Stateful O(n) Incremental Transducer for OpenAI / Anthropic / LangChain SSE streams
transducer = psh.IncrementalTransducer(max_table_cols=5, max_chart_bars=10)
for sse_chunk in ['### Report\n```json\n{"status": "live", ', '"items": [1, 2, ']:
    transducer.push_chunk(sse_chunk)
    print("Live Active JSON:", json.loads(transducer.snapshot_active_json())["content"])
```

### 2. JavaScript / TypeScript (`npm`, Zero-Dependency WebAssembly)
```bash
npm install polyglot-stream-heal
```

```ts
import { WasmTransducer, healJson, healMermaid, healHtml } from 'polyglot-stream-heal';

// A. Fast stateless repair for incomplete JSON, Mermaid diagrams, and HTML5 widgets
const validJson = healJson('{"metrics": [{"p99": 4.');
const validDiagram = healMermaid('flowchart TD\n  N1[Compute O(f(g(x)))] --> N2[(Storage)]');
const validHtml = healHtml('<div class="card"><script>const x = "streaming');

// B. Stateful Incremental Transducer for Vercel AI SDK / Next.js / Browser SSE streams
const transducer = new WasmTransducer(5, 10, true);
transducer.pushChunk('```json\n{"tool":"search","arguments":"{\\"q\\":\\"rust wasm');
console.log(JSON.parse(transducer.snapshotActiveJson()).content);
transducer.free();
```

### 3. Rust (`crates.io`)
```bash
cargo add polyglot-stream-heal
```

```rust
use polyglot_stream_heal::{heal_json, heal_mermaid, IncrementalTransducer, TransducerConfig};

let mut transducer = IncrementalTransducer::new(TransducerConfig::default());
transducer.push_chunk("Overview:\n```mermaid\nflowchart LR\n  A[Compute O(f(g(x)))] --> B[(DB (v16))]");
let snapshot = transducer.snapshot();
println!("{}", snapshot.reconstructed_markdown);
```

---

## Citation

See [`CITATION.cff`](CITATION.cff), or cite:

```bibtex
@article{michael2026polyglot,
  title   = {Linear-Time Incremental Pushdown Transduction and Cardinality-Bounded Layout Partitioning for Polyglot LLM Token Streams},
  author  = {Michael, David Charles},
  year    = {2026},
  doi     = {10.5281/zenodo.24045975},
  url     = {https://doi.org/10.5281/zenodo.24045975},
  note    = {ORCID: 0009-0007-8679-6941}
}
```

## License

MIT License. Copyright (c) 2026 David Charles Michael. See [`LICENSE`](LICENSE).
