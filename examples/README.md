# Runnable Examples & Peer-Reviewer Verification Suite (`examples/`)

This folder provides self-contained, runnable examples across all three target ecosystems (**Rust**, **Python `PyO3`**, and **Node.js `WebAssembly`**) demonstrating how `polyglot-stream-heal` repairs mid-stream truncated LLM outputs in sub-millisecond time.

## Files in This Folder

1. [`crates/polyglot-stream-heal/examples/demo.rs`](../crates/polyglot-stream-heal/examples/demo.rs): Native Rust example (`cargo run --example demo --release`).
2. [`examples/demo_python.py`](demo_python.py): Python `PyO3` C-extension example (`python3 examples/demo_python.py`).
3. [`examples/demo_node.mjs`](demo_node.mjs): Node.js WebAssembly example (`node examples/demo_node.mjs`).
4. [`examples/VERIFIED_OUTPUT.txt`](VERIFIED_OUTPUT.txt): Exact stdout captured from running the three examples.
5. [`examples/colab_quickstart.ipynb`](colab_quickstart.ipynb): Interactive Jupyter / Google Colab notebook.

## Running the Examples

```bash
cargo run -p polyglot-stream-heal --example demo --release
python3 examples/demo_python.py
node examples/demo_node.mjs
```
