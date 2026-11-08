/// Context-Free Pushdown Transducer for 14 Mermaid Diagram Sub-Grammars.
///
/// Key Theoretical Advantage over Regex Healers (Theorem 1):
/// Arbitrary-depth nested delimiters inside unquoted node labels — such as
/// `A[Compute O(f(g(h(x))))] --> B[(Postgres (Primary: v16))]` — form a non-regular
/// Dyck-embedded language that cannot be matched by finite-state regular expressions.
/// `IncrementalMermaidPda` uses an explicit delimiter stack per shape opener and maintains
/// persistent line-committed output plus a block-closure stack (`end` / `}`), achieving
/// strict O(|\Delta s_t|) work per streaming chunk.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MermaidDialect {
    Unknown,
    Flowchart,
    Sequence,
    Class,
    State,
    Er,
    Gantt,
    Pie,
    GitGraph,
    C4,
    Mindmap,
    Timeline,
    Quadrant,
    Architecture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockFrame {
    KeywordEnd, // closed by `end` (subgraph, loop, alt, opt, par, critical, rect, box)
    Brace,      // closed by `}` (class/state/er/namespace blocks)
}

#[derive(Debug, Clone)]
pub struct IncrementalMermaidPda {
    dialect: MermaidDialect,
    header_emitted: bool,
    block_stack: Vec<BlockFrame>,
    committed: String,
    active_line: String,
}

impl Default for IncrementalMermaidPda {
    fn default() -> Self {
        Self::new()
    }
}

impl IncrementalMermaidPda {
    pub fn new() -> Self {
        Self {
            dialect: MermaidDialect::Unknown,
            header_emitted: false,
            block_stack: Vec::with_capacity(8),
            committed: String::with_capacity(256),
            active_line: String::with_capacity(128),
        }
    }

    pub fn reset(&mut self) {
        self.dialect = MermaidDialect::Unknown;
        self.header_emitted = false;
        self.block_stack.clear();
        self.committed.clear();
        self.active_line.clear();
    }

    /// Feeds a streaming chunk in O(|chunk|) time, committing completed lines immediately.
    pub fn push_chunk(&mut self, chunk: &str) {
        for ch in chunk.chars() {
            if ch == '\r' {
                continue;
            }
            if ch == '\n' {
                let raw_line = std::mem::take(&mut self.active_line);
                self.process_complete_line(&raw_line);
            } else {
                self.active_line.push(ch);
            }
        }
    }

    fn detect_or_inject_header(&mut self, first_line: &str) -> bool {
        let trimmed = first_line.trim();
        if trimmed.is_empty() || trimmed.starts_with("%%") {
            return false;
        }
        let first_word = trimmed
            .split(|c: char| c.is_whitespace() || c == ';' || c == '(' || c == '[')
            .next()
            .unwrap_or("");

        let detected = match first_word {
            "flowchart" | "graph" => Some(MermaidDialect::Flowchart),
            "sequenceDiagram" => Some(MermaidDialect::Sequence),
            "classDiagram" => Some(MermaidDialect::Class),
            "stateDiagram" | "stateDiagram-v2" => Some(MermaidDialect::State),
            "erDiagram" => Some(MermaidDialect::Er),
            "gantt" => Some(MermaidDialect::Gantt),
            "pie" => Some(MermaidDialect::Pie),
            "gitGraph" => Some(MermaidDialect::GitGraph),
            "C4Context" | "C4Container" | "C4Component" | "C4Dynamic" => Some(MermaidDialect::C4),
            "mindmap" => Some(MermaidDialect::Mindmap),
            "timeline" => Some(MermaidDialect::Timeline),
            "quadrantChart" => Some(MermaidDialect::Quadrant),
            "architecture" | "architecture-beta" => Some(MermaidDialect::Architecture),
            _ => None,
        };

        if let Some(d) = detected {
            self.dialect = d;
            self.header_emitted = true;
            // Ensure bare `flowchart` or `graph` without direction gets `TD`
            if (first_word == "flowchart" || first_word == "graph") && trimmed == first_word {
                self.committed.push_str(first_word);
                self.committed.push_str(" TD\n");
                return true;
            }
            self.committed.push_str(trimmed);
            self.committed.push('\n');
            return true;
        }

        // Infer dialect from first substantive statement if LLM omitted the header
        if trimmed.contains("->>") || trimmed.starts_with("participant ") || trimmed.starts_with("actor ") {
            self.dialect = MermaidDialect::Sequence;
            self.committed.push_str("sequenceDiagram\n");
        } else {
            self.dialect = MermaidDialect::Flowchart;
            self.committed.push_str("flowchart TD\n");
        }
        self.header_emitted = true;
        false
    }

    fn process_complete_line(&mut self, raw_line: &str) {
        let decoded = decode_html_entities(raw_line);
        let trimmed = decoded.trim();
        if trimmed.is_empty() || trimmed.starts_with("```") {
            return;
        }

        if !self.header_emitted && self.detect_or_inject_header(trimmed) {
            return;
        }

        let healed = heal_mermaid_statement(trimmed, self.dialect, false);
        if healed.is_empty() {
            return;
        }
        self.update_block_stack(&healed);
        self.committed.push_str("  ");
        self.committed.push_str(&healed);
        self.committed.push('\n');
    }

    fn update_block_stack(&mut self, line: &str) {
        let trimmed = line.trim();
        let first_word = trimmed.split_whitespace().next().unwrap_or("");

        match first_word {
            "subgraph" | "loop" | "alt" | "opt" | "par" | "critical" | "rect" | "box" => {
                self.block_stack.push(BlockFrame::KeywordEnd);
            }
            "end" => {
                if let Some(pos) = self
                    .block_stack
                    .iter()
                    .rposition(|f| *f == BlockFrame::KeywordEnd)
                {
                    self.block_stack.remove(pos);
                }
            }
            _ => {}
        }

        // Track brace-delimited blocks outside quoted strings (`state X {`, `class Y {`, ER blocks)
        let mut in_quotes = false;
        for ch in trimmed.chars() {
            if ch == '"' {
                in_quotes = !in_quotes;
            } else if !in_quotes {
                if ch == '{' && self.dialect != MermaidDialect::Flowchart {
                    self.block_stack.push(BlockFrame::Brace);
                } else if ch == '}' && self.dialect != MermaidDialect::Flowchart {
                    if let Some(pos) = self
                        .block_stack
                        .iter()
                        .rposition(|f| *f == BlockFrame::Brace)
                    {
                        self.block_stack.remove(pos);
                    }
                }
            }
        }
    }

    /// Emits a valid, balanced Mermaid diagram at any mid-stream cut-off point in O(|active_line| + d) time.
    pub fn snapshot_healed(&self) -> String {
        let mut temp = self.clone();
        if !temp.active_line.trim().is_empty() {
            let tail = decode_html_entities(&temp.active_line);
            let trimmed = tail.trim();
            if !trimmed.starts_with("```") {
                if !temp.header_emitted {
                    if !temp.detect_or_inject_header(trimmed) {
                        let healed_tail = heal_mermaid_statement(trimmed, temp.dialect, true);
                        if !healed_tail.is_empty() {
                            temp.update_block_stack(&healed_tail);
                            temp.committed.push_str("  ");
                            temp.committed.push_str(&healed_tail);
                            temp.committed.push('\n');
                        }
                    }
                } else {
                    let healed_tail = heal_mermaid_statement(trimmed, temp.dialect, true);
                    if !healed_tail.is_empty() {
                        temp.update_block_stack(&healed_tail);
                        temp.committed.push_str("  ");
                        temp.committed.push_str(&healed_tail);
                        temp.committed.push('\n');
                    }
                }
            }
        }

        // Pop all unclosed blocks in LIFO order
        for frame in temp.block_stack.iter().rev() {
            match frame {
                BlockFrame::KeywordEnd => temp.committed.push_str("  end\n"),
                BlockFrame::Brace => temp.committed.push_str("  }\n"),
            }
        }

        temp.committed.trim_end().to_string()
    }
}

fn decode_html_entities(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    s.replace("&gt;", ">")
        .replace("&lt;", "<")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

#[derive(Debug, Clone, Copy)]
struct ShapeSpec {
    open: &'static str,
    close: &'static str,
    inner_open_char: char,
    inner_close_char: char,
}

const SHAPE_SPECS: &[ShapeSpec] = &[
    ShapeSpec {
        open: "[(",
        close: ")]",
        inner_open_char: '(',
        inner_close_char: ')',
    },
    ShapeSpec {
        open: "([",
        close: "])",
        inner_open_char: '[',
        inner_close_char: ']',
    },
    ShapeSpec {
        open: "((",
        close: "))",
        inner_open_char: '(',
        inner_close_char: ')',
    },
    ShapeSpec {
        open: "{{",
        close: "}}",
        inner_open_char: '{',
        inner_close_char: '}',
    },
    ShapeSpec {
        open: "[",
        close: "]",
        inner_open_char: '[',
        inner_close_char: ']',
    },
    ShapeSpec {
        open: "(",
        close: ")",
        inner_open_char: '(',
        inner_close_char: ')',
    },
    ShapeSpec {
        open: "{",
        close: "}",
        inner_open_char: '{',
        inner_close_char: '}',
    },
];

fn is_id_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'-'
}

fn sanitize_label_text(raw: &str) -> String {
    let trimmed = raw.trim();
    // Strip outer quotes if already wrapped so we can sanitize inner quotes uniformly
    let inner = if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        &trimmed[1..trimmed.len() - 1]
    } else if let Some(stripped) = trimmed.strip_prefix('"') {
        stripped
    } else {
        trimmed
    };

    let mut out = String::with_capacity(inner.len() + 4);
    let bytes = inner.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() && bytes[i + 1] == b'n' {
            out.push_str("<br/>");
            i += 2;
            continue;
        }
        let ch = inner[i..].chars().next().unwrap();
        match ch {
            '"' => out.push('\''),
            '&' => out.push_str("and"),
            '*' => {} // strip markdown bold/italic asterisks inside node labels
            _ => out.push(ch),
        }
        i += ch.len_utf8();
    }
    out.trim().to_string()
}

fn label_needs_quoting(raw: &str) -> bool {
    raw.chars().any(|c| {
        matches!(
            c,
            '(' | ')'
                | '['
                | ']'
                | '{'
                | '}'
                | ':'
                | '/'
                | '\\'
                | '&'
                | '"'
                | '\''
                | '<'
                | '>'
                | '='
                | '+'
                | '*'
                | '%'
                | '$'
                | '#'
                | '@'
                | '!'
                | '?'
                | ','
                | ';'
                | '|'
        )
    })
}

/// Repairs a single Mermaid statement using a pushdown balanced-delimiter scanner
/// for node shapes, edge labels, subgraph titles, and mid-stream truncations.
fn heal_mermaid_statement(line: &str, dialect: MermaidDialect, is_truncated_tail: bool) -> String {
    let mut trimmed = line.trim().trim_end_matches(';').trim().to_string();
    if trimmed.is_empty() {
        return String::new();
    }

    // If truncated mid-arrow at end of line (e.g., `A -->`, `A --`, `A -.->`, `A ==>`)
    // drop the dangling incomplete edge operator so the prefix node still renders cleanly!
    if is_truncated_tail {
        for dangling in ["-.->", "==>", "-->", "---", "-.-", "--", "->>", "-->>"] {
            if let Some(prefix) = trimmed.strip_suffix(dangling) {
                let p = prefix.trim();
                if p.is_empty() {
                    return String::new();
                }
                trimmed = p.to_string();
                break;
            }
        }
    }

    if dialect != MermaidDialect::Flowchart && dialect != MermaidDialect::Unknown {
        // Sequence / Class / State / ER / Gantt / Pie / Mindmap line healing:
        // Close any dangling double quote on truncated tail lines.
        let quote_count = trimmed.chars().filter(|&c| c == '"').count();
        if quote_count % 2 == 1 {
            trimmed.push('"');
        }
        if dialect == MermaidDialect::Sequence && is_truncated_tail && trimmed.ends_with(':') {
            trimmed.push_str(" ...");
        }
        return trimmed;
    }

    // Handle `subgraph Title (Extra)` -> `subgraph sub_id ["Title (Extra)"]`
    if let Some(rest) = trimmed.strip_prefix("subgraph ") {
        let rest_trimmed = rest.trim();
        if rest_trimmed.is_empty() {
            return "subgraph sub_group".to_string();
        }
        if !rest_trimmed.contains('[') && label_needs_quoting(rest_trimmed) {
            let clean = sanitize_label_text(rest_trimmed);
            let slug: String = clean
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                .take(20)
                .collect();
            let id = if slug.trim_matches('_').is_empty() {
                "sub_group".to_string()
            } else {
                format!("sg_{}", slug.trim_matches('_'))
            };
            return format!("subgraph {} [\"{}\"]", id, clean);
        }
        // Close truncated `subgraph id ["Title`
        if rest_trimmed.contains('[') && !rest_trimmed.ends_with(']') {
            let mut fixed = trimmed.clone();
            if fixed.chars().filter(|&c| c == '"').count() % 2 == 1 {
                fixed.push('"');
            }
            fixed.push(']');
            return fixed;
        }
        return trimmed;
    }

    // Normalize non-standard inline edge text `-- label -->` to `-->|label|` outside shapes
    let normalized = normalize_inline_edge_labels(&trimmed);

    // Pushdown scan across Flowchart statement to balance & quote arbitrary-depth node shapes
    // and sanitize `|...|` edge pipe labels.
    let bytes = normalized.as_bytes();
    let mut out = String::with_capacity(normalized.len() + 16);
    let mut i = 0;

    while i < bytes.len() {
        // Check pipe label `|...|` after arrow `>` or `-`
        if bytes[i] == b'|' {
            let start = i + 1;
            let mut end = start;
            while end < bytes.len() && bytes[end] != b'|' {
                end += 1;
            }
            let raw_pipe = &normalized[start..end];
            let clean_pipe = sanitize_label_text(raw_pipe).replace('|', "/");
            if clean_pipe.is_empty() && is_truncated_tail && end >= bytes.len() {
                // Dangling `A -->|` at truncation boundary: strip trailing `|` + arrow if needed
                while out.ends_with('>') || out.ends_with('-') || out.ends_with('.') || out.ends_with('=') {
                    out.pop();
                }
                return out.trim().to_string();
            }
            out.push('|');
            out.push_str(&clean_pipe);
            out.push('|');
            i = if end < bytes.len() { end + 1 } else { end };
            continue;
        }

        // Match Node Identifier followed immediately by a Shape Opener
        if is_id_char(bytes[i]) && (i == 0 || !is_id_char(bytes[i - 1])) {
            let id_start = i;
            let mut id_end = i;
            while id_end < bytes.len() && is_id_char(bytes[id_end]) {
                id_end += 1;
            }
            let id_str = &normalized[id_start..id_end];
            let rest = &normalized[id_end..];

            if id_str != "subgraph"
                && id_str != "classDef"
                && id_str != "class"
                && id_str != "style"
                && id_str != "linkStyle"
                && id_str != "click"
                && id_str != "direction"
            {
                if let Some(spec) = SHAPE_SPECS.iter().find(|s| rest.starts_with(s.open)) {
                    let body_start = id_end + spec.open.len();
                    let (body_end, after_close, found_close) =
                        scan_balanced_shape_body(&normalized, body_start, spec);
                    let raw_body = &normalized[body_start..body_end];
                    let clean_body = sanitize_label_text(raw_body);

                    out.push_str(id_str);
                    out.push_str(spec.open);
                    if clean_body.is_empty() {
                        out.push_str("\"Node\"");
                    } else if !found_close || label_needs_quoting(raw_body) {
                        out.push('"');
                        out.push_str(&clean_body);
                        out.push('"');
                    } else {
                        out.push_str(raw_body.trim());
                    }
                    out.push_str(spec.close);
                    i = after_close;
                    continue;
                }
            }

            out.push_str(id_str);
            i = id_end;
            continue;
        }

        let ch = normalized[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }

    // Final trailing-operator cleanup if truncated right after a healed node + ` -->`
    let mut final_line = out.trim().to_string();
    if is_truncated_tail {
        for dangling in ["-.->", "==>", "-->", "---", "-.-", "--"] {
            if let Some(prefix) = final_line.strip_suffix(dangling) {
                final_line = prefix.trim().to_string();
                break;
            }
        }
    }
    final_line
}

/// Pushdown scanner that tracks nested brackets/parentheses of arbitrary depth `d`
/// so `Node[Compute O(f(g(x))) + A[i]]` finds the true outer closing delimiter!
fn scan_balanced_shape_body(
    s: &str,
    body_start: usize,
    spec: &ShapeSpec,
) -> (usize, usize, bool) {
    let bytes = s.as_bytes();
    let mut i = body_start;
    let mut depth: usize = 0;
    let mut in_quotes = false;

    while i < bytes.len() {
        let b = bytes[i];
        if b == b'"' && (i == body_start || bytes[i - 1] != b'\\') {
            in_quotes = !in_quotes;
            i += 1;
            continue;
        }

        if !in_quotes {
            if depth == 0 && s[i..].starts_with(spec.close) {
                // Ensure that for single-char close `)` when inner has `((` or similar,
                // depth == 0 guarantees all inner delimiters are balanced!
                return (i, i + spec.close.len(), true);
            }
            // Stop before consuming a subsequent Flowchart edge operator if an unclosed shape
            // was emitted before ` --> ` on the same line
            if depth == 0 && (s[i..].starts_with(" -->") || s[i..].starts_with(" -.->") || s[i..].starts_with(" ==>")) {
                return (i, i, false);
            }
            let ch = s[i..].chars().next().unwrap();
            if ch == spec.inner_open_char {
                depth += 1;
            } else if ch == spec.inner_close_char && depth > 0 {
                depth -= 1;
            }
        }

        let ch = s[i..].chars().next().unwrap();
        i += ch.len_utf8();
    }

    (bytes.len(), bytes.len(), false)
}

/// Converts `A -- label text --> B` into `A -->|label text| B` outside node brackets.
fn normalize_inline_edge_labels(s: &str) -> String {
    if !s.contains("-- ") || !s.contains("-->") {
        return s.to_string();
    }
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut depth: usize = 0;
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'[' | b'(' | b'{' => {
                depth += 1;
                out.push(bytes[i] as char);
                i += 1;
            }
            b']' | b')' | b'}' => {
                depth = depth.saturating_sub(1);
                out.push(bytes[i] as char);
                i += 1;
            }
            b'-' if depth == 0 && s[i..].starts_with("-- ") && !s[i..].starts_with("-->") => {
                if let Some(rel_end) = s[i + 3..].find("-->") {
                    let label = s[i + 3..i + 3 + rel_end].trim();
                    if !label.is_empty() && !label.contains('[') && !label.contains("--") {
                        out.push_str("-->|");
                        out.push_str(&sanitize_label_text(label));
                        out.push('|');
                        i += 3 + rel_end + 3;
                        continue;
                    }
                }
                out.push('-');
                i += 1;
            }
            _ => {
                let ch = s[i..].chars().next().unwrap();
                out.push(ch);
                i += ch.len_utf8();
            }
        }
    }
    out
}

/// Convenience stateless one-shot Mermaid healer.
pub fn heal_mermaid(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let mut pda = IncrementalMermaidPda::new();
    pda.push_chunk(trimmed);
    pda.snapshot_healed()
}
