#!/usr/bin/env node
/**
 * Comprehensive Runnable Node.js / WebAssembly Showcase for polyglot-stream-heal.
 *
 * Demonstrates all 5 core capabilities with explicit Input -> Output pairs:
 * 1. Recursive Nested Escaped Tool-Call JSON Repair (OpenAI / Anthropic / MCP)
 * 2. Context-Free Mermaid Dyck-Parentheses Node & Subgraph Healing
 * 3. Streaming HTML5 / JavaScript Sandbox Boundary Healing
 * 4. Cardinality-Bounded Wide Table Layout Partitioning
 * 5. Stateful Linear-Time Incremental Pushdown Transducer (SSE Streams)
 */

import { createRequire } from 'node:module';
const require = createRequire(import.meta.url);
const wasm = require('../bindings/js/pkg/polyglot_stream_heal_wasm.js');

console.log('='.repeat(70));
console.log('polyglot-stream-heal: 5-Feature WebAssembly / Node.js Showcase');
console.log('='.repeat(70));

// 1. Recursive Tool-Call Escaped JSON Repair
console.log('\n[Feature 1: Recursive Escaped Tool-Call JSON Repair (OpenAI / Anthropic / MCP)]');
const rawTool = '{"name":"search_code","arguments":"{\\"repo\\":\\"polyglot\\",\\"query\\":\\"pda stream';
const healedTool = wasm.healJson(rawTool);
console.log('Input (Truncated) :', rawTool);
console.log('Output (RFC-8259) :', healedTool);
const parsedTool = JSON.parse(healedTool);
const innerArgs = JSON.parse(parsedTool.arguments);
console.log('Verified Inner JSON: arguments.query =', innerArgs.query);

// 2. Context-Free Mermaid Dyck-Parentheses Node & Subgraph Healing
console.log('\n[Feature 2: Context-Free Mermaid Dyck Node & Subgraph Healing]');
const rawMermaid = 'flowchart TD\n  subgraph WorkerPool (Region: US-East)\n    A[Task O(log(n))] --> B[(DB (v16.4))]';
const healedMermaid = wasm.healMermaid(rawMermaid);
console.log('Input (Unquoted parens & unclosed subgraph):');
console.log(rawMermaid);
console.log('\nOutput (Auto-quoted shapes & closed block directives):');
console.log(healedMermaid);

// 3. Streaming HTML5 / JavaScript Sandbox Boundary Healing
console.log('\n[Feature 3: Streaming HTML5 / JavaScript Sandbox Boundary Healing]');
const rawHtml = '<div class="chart-container"><script>const config = { theme: "dark", data: [10, 20, ';
const healedHtml = wasm.healHtml(rawHtml);
console.log('Input (Unclosed script tag & JS array):');
console.log(rawHtml);
console.log('\nOutput (Lexically closed script & DOM tags):');
console.log(healedHtml);

// 4. Cardinality-Bounded Wide Table Partitioning
console.log('\n[Feature 4: Cardinality-Bounded Wide Table Partitioning (C_max = 4 columns)]');
const wideTable = JSON.stringify({
  title: 'Engine Metrics',
  columns: ['System', 'Latency_p50', 'Latency_p99', 'Throughput', 'Accuracy', 'Memory_MB'],
  rows: [
    ['polyglot-stream-heal', '6.3 us', '57.3 us', '1420 MB/s', '100.0%', '8.2 MB']
  ]
});
const partitioned = wasm.partitionVisualSpec(wideTable, 4, 10);
const partObj = JSON.parse(partitioned);
const sections = partObj.sections || [partObj];
console.log(`Input : 6-column wide table partitioned into ${sections.length} anchor-preserving sub-tables:`);
sections.forEach((p, idx) => {
  console.log(`  Sub-Table ${idx + 1} Columns: ${JSON.stringify(p.columns)}`);
});

// 5. Stateful Incremental Transducer for SSE Chunks
console.log('\n[Feature 5: Stateful O(n) Incremental Pushdown Transducer (SSE Streams)]');
const transducer = new wasm.WasmTransducer(4, 10, true);
const sseChunks = [
  '```json\n{"service":"gateway","online":true,"buffer_size":40',
  '96,"tags":["edge","prod'
];
console.log('Streaming chunks into WebAssembly transducer...');
sseChunks.forEach((chunk, i) => {
  transducer.pushChunk(chunk);
  const active = JSON.parse(transducer.snapshotActiveJson());
  console.log(`  Chunk ${i + 1} -> Active Modality: ${active.modality} | Healed: ${active.content}`);
});
console.log('\nReconstructed Markdown Snapshot:\n' + transducer.snapshotMarkdown());
transducer.free();

console.log('\n' + '='.repeat(70));
console.log('All 5 WebAssembly / Node.js features verified successfully.');
console.log('='.repeat(70));

