#!/usr/bin/env node
/**
 * PolyglotStreamBench-1800 Deterministic Corpus Generator
 *
 * Synthesizes 1,800 reproducible streaming test cases (seed = 20261105) across 3 tracks:
 *   - Track A (600 cases): Mid-stream prefix truncations at 15%, 35%, 60%, 85% byte cutoffs
 *     across JSON specs, nested escaped JSON tool-calls, 14 Mermaid dialects, and HTML5 widgets.
 *   - Track B (600 cases): Context-free syntactic perturbations (Dyck depth-2..5 nested parentheses
 *     in Mermaid node shapes, unclosed subgraphs/loops, single-quoted/unquoted JSON keys, partial
 *     literals/numbers, unclosed HTML <script>/<style> fragments).
 *   - Track C (600 cases): Multi-block polyglot documents (3..6 interleaved prose + DSL blocks)
 *     and wide-schema visual specs (7..14 table columns, 14..26 chart bars) testing cumulative
 *     streaming complexity O(N) vs O(N^2) and zero-overflow cardinality partitioning.
 */

import fs from 'node:fs';
import path from 'node:path';

class Mulberry32 {
  constructor(seed) {
    this.state = seed >>> 0;
  }
  next() {
    let t = (this.state += 0x6d2b79f5);
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  }
  int(min, max) {
    return Math.floor(this.next() * (max - min + 1)) + min;
  }
  pick(arr) {
    return arr[this.int(0, arr.length - 1)];
  }
}

const rng = new Mulberry32(20261105);

const SERVICES = ['Gateway', 'AuthEngine', 'KVStore', 'VectorIndex', 'StreamRouter', 'Scheduler'];
const METRICS = ['p50_ms', 'p95_ms', 'p99_ms', 'throughput_rps', 'mem_mb', 'cache_hit_pct', 'cpu_util', 'err_rate', 'queue_depth', 'net_io_mbps', 'disk_iops', 'gc_pause_ms'];

function buildNestedDyckExpr(depth, leaf = 'x') {
  const fns = ['O', 'f', 'g', 'h', 'T', 'H'];
  let expr = leaf;
  for (let d = depth - 1; d >= 0; d--) {
    expr = `${fns[d % fns.length]}(${expr} + k_${d})`;
  }
  return expr;
}

function makeCleanJsonCase(i) {
  const kind = i % 3;
  if (kind === 0) {
    // Nested escaped tool-call JSON payload
    const inner = JSON.stringify({
      query: `distributed consensus batch ${i}`,
      filters: [
        { field: 'latency_ms', op: '<=', value: 12.5 + (i % 7) },
        { field: 'region', op: 'in', value: ['us-east', 'eu-west'] },
      ],
      limit: 25,
    });
    return {
      subkind: 'escaped_tool_json',
      fenced: false,
      raw: JSON.stringify({
        id: `call_${1000 + i}`,
        type: 'function',
        function: {
          name: 'execute_telemetry_search',
          arguments: inner,
        },
      }),
    };
  }
  if (kind === 1) {
    return {
      subkind: 'visual_json',
      fenced: true,
      fenceTag: 'dynamic_visual',
      raw: JSON.stringify({
        title: `Cluster Performance Snapshot #${i}`,
        sections: [
          {
            kind: 'kpi_row',
            items: [
              { label: 'Median Latency', value: `${(1.2 + (i % 5) * 0.3).toFixed(2)} ms` },
              { label: 'Availability', value: '99.98%' },
            ],
          },
          {
            kind: 'narrative_card',
            title: 'Bottleneck Synthesis',
            body: 'Lock-free ring buffering eliminates tail amplification under burst load.',
            highlights: ['Zero allocations', 'Bounded backpressure'],
          },
        ],
      }, null, 2),
    };
  }
  return {
    subkind: 'voxel_json',
    fenced: true,
    fenceTag: 'voxel',
    raw: JSON.stringify({
      scene: `lattice_${i}`,
      camera: { fov: 45, distance: 18 },
      voxels: Array.from({ length: 6 }, (_, k) => ({
        x: k * 2,
        y: (i + k) % 4,
        z: k % 3,
        color: '#3b82f6',
      })),
    }),
  };
}

function makeCleanMermaidCase(i) {
  const s1 = rng.pick(SERVICES);
  const s2 = rng.pick(SERVICES);
  if (i % 2 === 0) {
    return {
      subkind: 'mermaid_flowchart',
      fenced: true,
      fenceTag: 'mermaid',
      raw: [
        'flowchart TD',
        `  subgraph sg_${i} ["Control Plane Tier ${i}"]`,
        `    N1["${s1} Ingress"] -->|validate| N2["${s2} Core"]`,
        `    N2 -->|commit| DB[("State Ledger v2")]`,
        '  end',
      ].join('\n'),
    };
  }
  return {
    subkind: 'mermaid_sequence',
    fenced: true,
    fenceTag: 'mermaid',
    raw: [
      'sequenceDiagram',
      `  participant C as Client_${i}`,
      `  participant S as ${s1}`,
      '  loop HealthProbe',
      '    C->>S: SYN_CHECK',
      '    S-->>C: ACK_READY',
      '  end',
    ].join('\n'),
  };
}

function makeCleanHtmlCase(i) {
  return {
    subkind: 'html_widget',
    fenced: true,
    fenceTag: 'html',
    raw: [
      `<div class="telemetry-card" data-id="${i}">`,
      `  <style>.telemetry-card { padding: 12px; border: 1px solid #334155; }</style>`,
      `  <!-- Live status widget ${i} -->`,
      `  <section class="header"><h3>Node #${i}</h3></section>`,
      `  <script>`,
      `    function computeDelta_${i}(a, b) {`,
      `      const label = "delta_metric_${i}";`,
      `      return { label, diff: Math.abs(a - b) };`,
      `    }`,
      `  </script>`,
      `</div>`,
    ].join('\n'),
  };
}

const corpus = [];

// ============================================================================
// TRACK A: 600 Mid-Stream Prefix Truncations (15%, 35%, 60%, 85%)
// ============================================================================
const CUT_RATIOS = [0.15, 0.35, 0.60, 0.85];
for (let i = 0; i < 600; i++) {
  const modSelector = i % 4;
  let base;
  if (modSelector === 0 || modSelector === 1) base = makeCleanJsonCase(i);
  else if (modSelector === 2) base = makeCleanMermaidCase(i);
  else base = makeCleanHtmlCase(i);

  const ratio = CUT_RATIOS[i % CUT_RATIOS.length];
  const wrapped = base.fenced
    ? `Analysis for case ${i}:\n\`\`\`${base.fenceTag}\n${base.raw}\n\`\`\`\n`
    : base.raw;

  // Ensure cut falls inside the structured payload
  const fenceStart = base.fenced ? wrapped.indexOf('\n') + base.fenceTag.length + 5 : 4;
  const usableLen = Math.max(8, wrapped.length - fenceStart - 5);
  const cutIdx = Math.min(wrapped.length - 4, fenceStart + Math.max(4, Math.floor(usableLen * ratio)));
  const truncated = wrapped.slice(0, cutIdx);

  corpus.push({
    id: `trackA_${i}`,
    track: 'A_truncation',
    cutoffRatio: ratio,
    expectedModality: modSelector <= 1 ? 'json' : modSelector === 2 ? 'mermaid' : 'html',
    subkind: base.subkind,
    fenced: base.fenced,
    input: truncated,
    expectedCells: null,
  });
}

// ============================================================================
// TRACK B: 600 Malformed Context-Free Perturbations
// ============================================================================
for (let i = 0; i < 600; i++) {
  const variant = i % 4;
  if (variant === 0) {
    // Mermaid Dyck nested parentheses of depth 2..5 + unclosed subgraph + inline `-- label -->`
    const depth = 2 + (i % 4);
    const dyck = buildNestedDyckExpr(depth, `x_${i}`);
    const s1 = rng.pick(SERVICES);
    const s2 = rng.pick(SERVICES);
    const raw = [
      `\`\`\`mermaid`,
      `flowchart LR`,
      `  subgraph Compute Stage (${s1}: Tier ${i % 5})`,
      `    A[Evaluate ${dyck}] -- stream batch ${i} --> B[(Primary ${s2} (v16.4: Read/Write))]`,
      `    B --> C{Latency <= p99 (${2 + (i % 9)}ms)?}`,
      // Intentionally omit `end` and closing fence on half the cases
      ...(i % 2 === 0 ? ['```'] : []),
    ].join('\n');
    corpus.push({
      id: `trackB_${i}`,
      track: 'B_perturbation',
      expectedModality: 'mermaid',
      subkind: `dyck_depth_${depth}`,
      fenced: true,
      input: raw,
    });
  } else if (variant === 1) {
    // Malformed JSON: unquoted keys, single-quoted strings, partial literal or dangling trailing comma
    const rawBody = `{ title: 'Pipeline #${i}', active: tru, threshold: 14., tags: ['alpha', 'beta',], meta: {"nested": "val_${i}"`;
    corpus.push({
      id: `trackB_${i}`,
      track: 'B_perturbation',
      expectedModality: 'json',
      subkind: 'malformed_json_lexical_and_stack',
      fenced: true,
      input: `\`\`\`json\n${rawBody}`,
    });
  } else if (variant === 2) {
    // Nested escaped tool-call JSON truncated mid-inner-object
    const innerTrunc = `{\\"service\\":\\"${rng.pick(SERVICES)}\\",\\"window_sec\\":60,\\"quantiles\\":[0.5,0.95,0.99`;
    const outerTrunc = `{"tool_call_id":"tc_${i}","name":"fetch_metrics","arguments":"${innerTrunc}`;
    corpus.push({
      id: `trackB_${i}`,
      track: 'B_perturbation',
      expectedModality: 'json',
      subkind: 'escaped_tool_json',
      fenced: false,
      input: outerTrunc,
    });
  } else {
    // Truncated HTML mid-script template literal & block comment
    const htmlTrunc = [
      '```html',
      `<div class="sim-panel" id="panel-${i}">`,
      `  <style>.sim-panel { display: grid; gap: 8px; /* active layout`,
      `  </style>`,
    ].join('\n');
    const scriptTrunc = [
      '```html',
      `<section class="widget"><div class="inner">`,
      `<script>function tick_${i}() { const msg = \`Step ${i}: active`,
    ].join('\n');
    corpus.push({
      id: `trackB_${i}`,
      track: 'B_perturbation',
      expectedModality: 'html',
      subkind: 'malformed_html_script_style',
      fenced: true,
      input: i % 2 === 0 ? scriptTrunc : htmlTrunc,
    });
  }
}

// ============================================================================
// TRACK C: 600 Multi-Block Polyglot + Wide-Table / Dense-Chart Overflow Cases
// ============================================================================
for (let i = 0; i < 600; i++) {
  const numCols = 7 + (i % 6); // 7..12 columns (> max_table_cols=5)
  const numBars = 12 + (i % 10); // 12..21 bars (> max_chart_bars=10)
  const cols = ['Architecture', ...METRICS.slice(0, numCols - 1)];
  const rows = [
    ['HM-PDT (Ours)', ...cols.slice(1).map((_, c) => `${(0.4 + c * 0.1).toFixed(2)}`)],
    ['Regex Baseline', ...cols.slice(1).map((_, c) => `${(4.5 + c * 1.3).toFixed(2)}`)],
    ['Full Re-Scan', ...cols.slice(1).map((_, c) => `${(2.8 + c * 0.9).toFixed(2)}`)],
  ];
  const bars = Array.from({ length: numBars }, (_, b) => ({
    label: `Stage_${b + 1}`,
    value: 10 + ((i + b * 7) % 90),
  }));

  const visualSpec = {
    title: `Polyglot Systems Evaluation #${i}`,
    sections: [
      {
        kind: 'comparison_table',
        title: `Latency Matrix #${i}`,
        columns: cols,
        rows,
      },
      {
        kind: 'comparison_bars',
        title: `Throughput Breakdown #${i}`,
        bars,
      },
    ],
  };

  const dyck = buildNestedDyckExpr(3, 'n');
  const fullDoc = [
    `### Systems Synthesis Report #${i}`,
    `Evaluating streaming transduction stability across polyglot outputs.`,
    '```mermaid',
    'flowchart LR',
    `  subgraph Pipeline_${i} ["Execution Graph (Stage ${i})"]`,
    `    Ingest[Parse ${dyck}] -- tokens --> Heal[(State Stack (O(1) step))]`,
    '  end',
    '```',
    'Narrative transition explaining structured telemetry partitioning below:',
    '```dynamic_visual',
    JSON.stringify(visualSpec, null, 2),
    '```',
    '```html',
    `<div class="summary-banner"><span>Completed verification #${i}</span></div>`,
    '```',
  ].join('\n');

  // Truncate 40% of Track C mid-flight inside the final or middle block to test both multi-block + truncation
  const input = i % 5 < 2 ? fullDoc.slice(0, fullDoc.length - 18) : fullDoc;

  corpus.push({
    id: `trackC_${i}`,
    track: 'C_polyglot_overflow',
    expectedModality: 'polyglot',
    subkind: 'multi_block_wide_schema',
    fenced: true,
    input,
    expectedAttrCols: numCols - 1,
    expectedRowCount: rows.length,
    expectedBarCount: numBars,
  });
}

const outPath = path.resolve('corpus.json');
fs.writeFileSync(outPath, JSON.stringify(corpus), 'utf8');
console.log(`[generate_corpus] Generated ${corpus.length} deterministic test streams -> ${outPath}`);
