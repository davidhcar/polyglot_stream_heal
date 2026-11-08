#!/usr/bin/env node
/**
 * Runnable Node.js / WebAssembly Example for polyglot-stream-heal
 */

import { createRequire } from 'node:module';
const require = createRequire(import.meta.url);
const wasm = require('../bindings/js/pkg/polyglot_stream_heal_wasm.js');

console.log('=== 1. Incremental SSE Chunk Healing (WebAssembly / npm) ===');
const transducer = new wasm.WasmTransducer(4, 10, true);
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
console.log('Healed:', wasm.healHtml(rawHtml));
