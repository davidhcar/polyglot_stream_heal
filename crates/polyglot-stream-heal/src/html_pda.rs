/// Incremental Streaming HTML5 / CSS / JS Sandbox Boundary Healer.
///
/// Tracks open container tags, comments, `<script>`, and `<style>` regions incrementally
/// in O(|\Delta s_t|) time so truncated mid-stream HTML widgets render safely without
/// swallowing subsequent DOM nodes or throwing syntax errors inside inline scripts.

#[derive(Debug, Clone)]
pub struct IncrementalHtmlPda {
    buffer: String,
    open_tags: Vec<String>,
    scan_cursor: usize,
}

impl Default for IncrementalHtmlPda {
    fn default() -> Self {
        Self::new()
    }
}

const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

impl IncrementalHtmlPda {
    pub fn new() -> Self {
        Self {
            buffer: String::with_capacity(512),
            open_tags: Vec::with_capacity(16),
            scan_cursor: 0,
        }
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
        self.open_tags.clear();
        self.scan_cursor = 0;
    }

    pub fn push_chunk(&mut self, chunk: &str) {
        self.buffer.push_str(chunk);
        self.advance_tag_scanner();
    }

    /// Incrementally advances `scan_cursor` over completed `<...>` tags outside `<script>`/`<style>`/`<!--`.
    fn advance_tag_scanner(&mut self) {
        let bytes = self.buffer.as_bytes();
        let mut i = self.scan_cursor;

        while i < bytes.len() {
            if bytes[i] != b'<' {
                i += 1;
                self.scan_cursor = i;
                continue;
            }
            self.scan_cursor = i;

            // Wait until a complete `>` or `-->` arrives before advancing past `<`
            if self.buffer[i..].starts_with("<!--") {
                if let Some(end_rel) = self.buffer[i + 4..].find("-->") {
                    i += 4 + end_rel + 3;
                    self.scan_cursor = i;
                    continue;
                } else {
                    break;
                }
            }

            let Some(gt_rel) = self.buffer[i..].find('>') else {
                break;
            };

            let tag_slice = &self.buffer[i + 1..i + gt_rel].trim();
            if tag_slice.starts_with('!') || tag_slice.starts_with('?') || tag_slice.is_empty() {
                i += gt_rel + 1;
                self.scan_cursor = i;
                continue;
            }

            let is_closing = tag_slice.starts_with('/');
            let self_closing = tag_slice.ends_with('/');
            let name_src = if is_closing { &tag_slice[1..] } else { tag_slice };
            let tag_name: String = name_src
                .trim_start()
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
                .collect::<String>()
                .to_ascii_lowercase();

            if tag_name.is_empty() {
                i += gt_rel + 1;
                self.scan_cursor = i;
                continue;
            }

            // If entering `<script>` or `<style>`, wait for matching `</script>` or `</style>`
            if !is_closing && !self_closing && (tag_name == "script" || tag_name == "style") {
                let close_pat = if tag_name == "script" {
                    "</script>"
                } else {
                    "</style>"
                };
                let after_open = i + gt_rel + 1;
                if let Some(close_rel) = self.buffer[after_open..]
                    .to_ascii_lowercase()
                    .find(close_pat)
                {
                    i = after_open + close_rel + close_pat.len();
                    self.scan_cursor = i;
                    continue;
                } else {
                    // Leave cursor at `<script` or `<style` so `snapshot_healed()` balances its body!
                    break;
                }
            }

            if is_closing {
                if let Some(pos) = self.open_tags.iter().rposition(|t| t == &tag_name) {
                    self.open_tags.truncate(pos);
                }
            } else if !self_closing && !VOID_ELEMENTS.contains(&tag_name.as_str()) {
                self.open_tags.push(tag_name);
            }

            i += gt_rel + 1;
            self.scan_cursor = i;
        }
    }

    pub fn snapshot_healed(&self) -> String {
        if self.buffer.trim().is_empty() {
            return String::new();
        }

        // Preserve exact byte alignment with `self.scan_cursor` before trimming!
        let mut out = self.buffer.clone();
        let tail_start = self.scan_cursor.min(out.len());
        let tail = &out[tail_start..];

        if tail.starts_with("<!--") && !tail.contains("-->") {
            out.push_str(" -->");
        } else if tail.to_ascii_lowercase().starts_with("<script") {
            if let Some(gt_rel) = tail.find('>') {
                let script_body = &tail[gt_rel + 1..];
                let closer = close_js_fragment(script_body);
                out.push_str(&closer);
                out.push_str("\n</script>");
            } else {
                // Opening `<script ...` tag was cut off mid-attribute
                out.truncate(tail_start);
            }
        } else if tail.to_ascii_lowercase().starts_with("<style") {
            if let Some(gt_rel) = tail.find('>') {
                let style_body = &tail[gt_rel + 1..];
                let closer = close_css_fragment(style_body);
                out.push_str(&closer);
                out.push_str("\n</style>");
            } else {
                out.truncate(tail_start);
            }
        } else if tail.starts_with('<') && !tail.contains('>') {
            // Strip incomplete dangling opening/closing `<div class="...` or `</div` tag at byte cutoff
            out.truncate(tail_start);
        }

        // Close all open container elements in LIFO order
        for tag in self.open_tags.iter().rev() {
            out.push_str("</");
            out.push_str(tag);
            out.push('>');
        }

        out.trim().to_string()
    }
}

fn close_js_fragment(script_body: &str) -> String {
    let mut in_single = false;
    let mut in_double = false;
    let mut in_template = false;
    let mut in_block_comment = false;
    let mut in_line_comment = false;
    let mut escape = false;
    let mut delim_stack: Vec<char> = Vec::new();

    let bytes = script_body.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if in_line_comment {
            if b == b'\n' {
                in_line_comment = false;
            }
            i += 1;
            continue;
        }
        if in_block_comment {
            if b == b'*' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                in_block_comment = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        if escape {
            escape = false;
            i += 1;
            continue;
        }
        if b == b'\\' && (in_single || in_double || in_template) {
            escape = true;
            i += 1;
            continue;
        }
        if !in_single && !in_double && !in_template {
            if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                in_line_comment = true;
                i += 2;
                continue;
            }
            if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
                in_block_comment = true;
                i += 2;
                continue;
            }
        }
        if b == b'\'' && !in_double && !in_template {
            in_single = !in_single;
        } else if b == b'"' && !in_single && !in_template {
            in_double = !in_double;
        } else if b == b'`' && !in_single && !in_double {
            in_template = !in_template;
        } else if !in_single && !in_double && !in_template {
            match b {
                b'{' => delim_stack.push('}'),
                b'(' => delim_stack.push(')'),
                b'[' => delim_stack.push(']'),
                b'}' | b')' | b']' => {
                    if delim_stack.last() == Some(&(b as char)) {
                        delim_stack.pop();
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }

    let mut suffix = String::new();
    if in_block_comment {
        suffix.push_str(" */");
    }
    if in_single {
        suffix.push('\'');
    }
    if in_double {
        suffix.push('"');
    }
    if in_template {
        suffix.push('`');
    }
    for closer in delim_stack.into_iter().rev() {
        suffix.push(closer);
    }
    suffix
}

fn close_css_fragment(style_body: &str) -> String {
    let mut in_comment = false;
    let mut open_braces: usize = 0;
    let bytes = style_body.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if in_comment {
            if bytes[i] == b'*' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                in_comment = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            in_comment = true;
            i += 2;
            continue;
        }
        if bytes[i] == b'{' {
            open_braces += 1;
        } else if bytes[i] == b'}' {
            open_braces = open_braces.saturating_sub(1);
        }
        i += 1;
    }

    let mut suffix = String::new();
    if in_comment {
        suffix.push_str(" */");
    }
    for _ in 0..open_braces {
        suffix.push('}');
    }
    suffix
}

pub fn heal_html(raw: &str) -> String {
    let mut pda = IncrementalHtmlPda::new();
    pda.push_chunk(raw);
    pda.snapshot_healed()
}
