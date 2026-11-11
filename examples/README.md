# Runnable Examples & Verification Suite (`examples/`)

This directory provides self-contained, runnable examples across all three target ecosystems (**Rust**, **Python `PyO3`**, and **Node.js `WebAssembly`**) demonstrating the 5 core capabilities of `polyglot-stream-heal`.

---

## The 5 Core Capabilities & Verification Matrix

| Capability | Problem Solved | Input (Truncated / Perturbed) | Output (Healed / RFC-Compliant) |
| :--- | :--- | :--- | :--- |
| **1. Recursive Tool-Call JSON** | Solves broken JSON when model truncates mid-escaped tool argument string. | `{"name":"search","arguments":"{\"query\":\"rust pda` | `{"name":"search","arguments":"{\"query\":\"rust pda\"}"}` |
| **2. Context-Free Mermaid Dyck** | Quotes unquoted nested parens like `O(f(g(x)))` & closes nested `subgraph` blocks. | `flowchart TD\n subgraph DB\n A[Task O(f(g(x)))] --> B[(Log` | `flowchart TD\n subgraph DB\n A["Task O(f(g(x)))"] --> B[("Log")]\n end` |
| **3. Streaming HTML5 / JS Sandbox** | Closes unclosed `<script>`, `<style>`, template literals, and DOM tags. | `<div class="card"><script>const val = "stream` | `<div class="card"><script>const val = "stream";</script></div>` |
| **4. Cardinality Layout Partition** | Splits 7+ col tables into stacked sub-tables without horizontal overflow. | Table schema with 7 columns $\times$ 10 rows ($C > C_{\max}$) | 2 vertically stacked sub-tables ($C \le 4$), **anchored on Entity Col 0**. |
| **5. Linear-Time SSE Transducer** | Maintains persistent state across incoming chunks in $\Theta(N)$ optimal time. | Sequence of 15–40 ms SSE byte packets | Instant valid Markdown snapshot at any byte cutoff ($p50 = 6.3\,\mu\text{s}$). |

---

## Files in This Folder

1. [`examples/demo_python.py`](demo_python.py): Python `PyO3` native C-extension example (`python3 examples/demo_python.py`).
2. [`examples/demo_node.mjs`](demo_node.mjs): Node.js / WebAssembly example (`node examples/demo_node.mjs`).
3. [`crates/polyglot-stream-heal/examples/demo.rs`](../crates/polyglot-stream-heal/examples/demo.rs): Native safe Rust binary (`cargo run --example demo --release`).
4. [`examples/colab_quickstart.ipynb`](colab_quickstart.ipynb): Interactive Jupyter / Google Colab notebook for live browser execution.
5. [`examples/VERIFIED_OUTPUT.txt`](VERIFIED_OUTPUT.txt): Captured stdout from running the 5-feature test suite.

---

## Running the Examples

```bash
# Python (PyPI)
python3 examples/demo_python.py

# Node.js (npm / WebAssembly)
node examples/demo_node.mjs

# Rust (crates.io)
cargo run -p polyglot-stream-heal --example demo --release
```

