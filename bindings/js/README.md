# `polyglot-stream-heal` (JavaScript / TypeScript WebAssembly, `npm`)

**Ultra-fast $6.3\,\mu\text{s}$ incremental $O(n)$ streaming JSON repair, 14-dialect Mermaid diagram healer, and HTML5 sandbox balancer for LLM token streams (WebAssembly + TypeScript alternative to `partial-json` and `jsonrepair`).**

[![npm](https://img.shields.io/npm/v/polyglot-stream-heal.svg)](https://www.npmjs.com/package/polyglot-stream-heal)
[![PyPI](https://img.shields.io/pypi/v/polyglot-stream-heal.svg)](https://pypi.org/project/polyglot-stream-heal/)
[![Crates.io](https://img.shields.io/crates/v/polyglot-stream-heal.svg)](https://crates.io/crates/polyglot-stream-heal)

- **Author:** **David Charles Michael** (`davidchar@gmail.com` | [ORCID: `0009-0007-8679-6941`](https://orcid.org/0009-0007-8679-6941))
- **License:** MIT
- **Zero-Dependency WebAssembly (`181.5 KB` `.wasm`, `85 KB` gzipped):** Compiled from Rust via `wasm-bindgen` with full TypeScript `.d.ts` type declarations and zero `preinstall`/`postinstall` scripts (100% compliant with strict npm v12 install-time security). Designed for Next.js, Vercel AI SDK, LangChain.js, Edge Workers, and browser streaming UIs.

---

## 1. Installation

```bash
npm install polyglot-stream-heal
```

---

## 2. How to Implement & Verified Output

### Runnable Code Snippet (`node examples/demo_node.mjs`)

```javascript
import { WasmTransducer, healHtml, healJson, healMermaid } from 'polyglot-stream-heal';

console.log('=== 1. Incremental SSE Chunk Healing (WebAssembly / npm) ===');
const transducer = new WasmTransducer(4, 10, true);
const sseChunks = [
  '```dynamic_visual\n{"title":"Wide Latency Matrix",',
  '"columns":["Engine","C1","C2","C3","C4","C5","C6"],',
  '"rows":[["HM-PDT","1.1","1.2","1.3","1.4","1.5","1.6"]]',
];
for (const chunk of sseChunks) {
  transducer.pushChunk(chunk);
}
const active = JSON.parse(transducer.snapshotActiveJson());
console.log('Modality       :', active.modality);
console.log('Was Partitioned:', active.was_partitioned);
console.log('Healed Payload :', active.content);
transducer.free();

console.log('\n=== 2. Truncated HTML5 / Script Boundary Healing ===');
const rawHtml = '<div class="widget"><script>function calc(x) { const s = "live_value';
console.log('Input :', rawHtml);
console.log('Healed:', healHtml(rawHtml));
```

### Exact Verified Output (`examples/VERIFIED_OUTPUT.txt`)

```text
=== 1. Incremental SSE Chunk Healing (WebAssembly / npm) ===
Modality       : dynamic_visual
Was Partitioned: true
Healed Payload : {"layout":"adaptive_vertical_stack","partitioned":true,"sections":[{"columns":["Engine","C1","C2","C3"],"partitionIndex":1,"partitionTotal":2,"rows":[["HM-PDT","1.1","1.2","1.3"]],"title":"Wide Latency Matrix (Part 1/2)"},{"columns":["Engine","C4","C5","C6"],"partitionIndex":2,"partitionTotal":2,"rows":[["HM-PDT","1.4","1.5","1.6"]],"title":"Wide Latency Matrix (Part 2/2)"}]}

=== 2. Truncated HTML5 / Script Boundary Healing ===
Input : <div class="widget"><script>function calc(x) { const s = "live_value
Healed: <div class="widget"><script>function calc(x) { const s = "live_value"}
</script></div>
```
