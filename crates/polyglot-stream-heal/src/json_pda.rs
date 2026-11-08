/// Incremental Context-Free Pushdown Automaton for Streaming JSON & Nested Escaped JSON Repair.
///
/// Guarantees:
/// 1. Strict O(|\Delta s_t|) work per appended token chunk (zero full-buffer re-scanning).
/// 2. O(d) closure emission at any arbitrary mid-token byte truncation offset, where d is active
///    nesting depth.
/// 3. Recursive healing of escaped JSON payloads inside string values (e.g., OpenAI/Anthropic
///    streaming tool-call `arguments` strings like `"{\"query\": \"distributed"`).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContainerState {
    ObjectExpectKeyOrEnd,
    ObjectExpectColon,
    ObjectExpectValue,
    ObjectAfterValue,
    ArrayExpectValueOrEnd,
    ArrayAfterValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LexicalState {
    Normal,
    InString { is_key: bool },
    InStringEscape { is_key: bool },
    InUnicodeEscape { is_key: bool, remaining_hex: u8 },
    InSingleQuoteString { is_key: bool },
    InSingleQuoteEscape { is_key: bool },
    InNumber { saw_digit_after_marker: bool },
    InIdentifier { first_char: char, len: usize },
    InUnquotedKey,
}

#[derive(Debug, Clone)]
pub struct IncrementalJsonPda {
    stack: Vec<ContainerState>,
    lex: LexicalState,
    committed: String,
    /// Staging buffer for an in-flight key `"key": ` so if truncation occurs before any value
    /// starts, or mid-key, we can deterministically complete or drop dangling keys in O(1)
    /// without scanning backwards through `committed`.
    pending_member_prefix: String,
    in_pending_member: bool,
    /// Captures current string body when scanning a value string so escaped inner JSON
    /// (`"{\"a\": 1"`) can be recursively healed on closure.
    active_value_string_raw: String,
    value_string_start_idx: Option<usize>,
    saw_root_token: bool,
    repair_nested_escaped_json: bool,
}

impl Default for IncrementalJsonPda {
    fn default() -> Self {
        Self::new(true)
    }
}

impl IncrementalJsonPda {
    pub fn new(repair_nested_escaped_json: bool) -> Self {
        Self {
            stack: Vec::with_capacity(16),
            lex: LexicalState::Normal,
            committed: String::with_capacity(256),
            pending_member_prefix: String::with_capacity(64),
            in_pending_member: false,
            active_value_string_raw: String::new(),
            value_string_start_idx: None,
            saw_root_token: false,
            repair_nested_escaped_json,
        }
    }

    pub fn reset(&mut self) {
        self.stack.clear();
        self.lex = LexicalState::Normal;
        self.committed.clear();
        self.pending_member_prefix.clear();
        self.in_pending_member = false;
        self.active_value_string_raw.clear();
        self.value_string_start_idx = None;
        self.saw_root_token = false;
    }

    #[inline]
    pub fn is_balanced(&self) -> bool {
        self.saw_root_token
            && self.stack.is_empty()
            && matches!(
                self.lex,
                LexicalState::Normal | LexicalState::InNumber { saw_digit_after_marker: true }
            )
            && !self.in_pending_member
    }

    #[inline]
    fn push_out(&mut self, ch: char) {
        if self.in_pending_member {
            self.pending_member_prefix.push(ch);
        } else {
            self.committed.push(ch);
        }
    }

    #[inline]
    fn push_str_out(&mut self, s: &str) {
        if self.in_pending_member {
            self.pending_member_prefix.push_str(s);
        } else {
            self.committed.push_str(s);
        }
    }

    #[inline]
    fn flush_pending_member(&mut self) {
        if self.in_pending_member {
            self.committed.push_str(&self.pending_member_prefix);
            self.pending_member_prefix.clear();
            self.in_pending_member = false;
        }
    }

    fn finish_value_on_stack(&mut self) {
        if let Some(top) = self.stack.last_mut() {
            match top {
                ContainerState::ObjectExpectValue => *top = ContainerState::ObjectAfterValue,
                ContainerState::ArrayExpectValueOrEnd => *top = ContainerState::ArrayAfterValue,
                _ => {}
            }
        }
    }

    /// Feeds a streaming chunk in O(|chunk|) time.
    pub fn push_chunk(&mut self, chunk: &str) {
        for ch in chunk.chars() {
            self.step(ch);
        }
    }

    fn step(&mut self, ch: char) {
        match self.lex {
            LexicalState::InString { is_key } => {
                if ch == '\\' {
                    self.push_out('\\');
                    if !is_key {
                        self.active_value_string_raw.push('\\');
                    }
                    self.lex = LexicalState::InStringEscape { is_key };
                } else if ch == '"' {
                    if !is_key && self.repair_nested_escaped_json {
                        self.maybe_heal_completed_escaped_json();
                    }
                    self.push_out('"');
                    self.value_string_start_idx = None;
                    self.active_value_string_raw.clear();
                    self.lex = LexicalState::Normal;
                    if is_key {
                        if let Some(top) = self.stack.last_mut() {
                            *top = ContainerState::ObjectExpectColon;
                        }
                    } else {
                        self.finish_value_on_stack();
                    }
                } else if (ch as u32) < 0x20 {
                    // Escape raw control characters (literal newlines/tabs inside LLM strings)
                    let escaped = match ch {
                        '\n' => "\\n",
                        '\r' => "\\r",
                        '\t' => "\\t",
                        _ => " ",
                    };
                    self.push_str_out(escaped);
                    if !is_key {
                        self.active_value_string_raw.push_str(escaped);
                    }
                } else {
                    self.push_out(ch);
                    if !is_key {
                        self.active_value_string_raw.push(ch);
                    }
                }
            }
            LexicalState::InStringEscape { is_key } => {
                if ch == 'u' {
                    self.push_out('u');
                    if !is_key {
                        self.active_value_string_raw.push('u');
                    }
                    self.lex = LexicalState::InUnicodeEscape {
                        is_key,
                        remaining_hex: 4,
                    };
                } else {
                    // Normalize invalid JSON escapes like \' -> '
                    if matches!(ch, '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't') {
                        self.push_out(ch);
                        if !is_key {
                            self.active_value_string_raw.push(ch);
                        }
                    } else {
                        // Pop the preceding backslash and emit literal character safely
                        if self.in_pending_member {
                            self.pending_member_prefix.pop();
                        } else {
                            self.committed.pop();
                        }
                        if !is_key {
                            self.active_value_string_raw.pop();
                            self.active_value_string_raw.push(ch);
                        }
                        self.push_out(ch);
                    }
                    self.lex = LexicalState::InString { is_key };
                }
            }
            LexicalState::InUnicodeEscape {
                is_key,
                remaining_hex,
            } => {
                if ch.is_ascii_hexdigit() {
                    self.push_out(ch);
                    if !is_key {
                        self.active_value_string_raw.push(ch);
                    }
                    if remaining_hex <= 1 {
                        self.lex = LexicalState::InString { is_key };
                    } else {
                        self.lex = LexicalState::InUnicodeEscape {
                            is_key,
                            remaining_hex: remaining_hex - 1,
                        };
                    }
                } else {
                    // Pad incomplete unicode escape and reprocess character in string mode
                    for _ in 0..remaining_hex {
                        self.push_out('0');
                        if !is_key {
                            self.active_value_string_raw.push('0');
                        }
                    }
                    self.lex = LexicalState::InString { is_key };
                    self.step(ch);
                }
            }
            LexicalState::InSingleQuoteString { is_key } => {
                if ch == '\\' {
                    self.lex = LexicalState::InSingleQuoteEscape { is_key };
                } else if ch == '\'' {
                    self.push_out('"');
                    self.value_string_start_idx = None;
                    self.active_value_string_raw.clear();
                    self.lex = LexicalState::Normal;
                    if is_key {
                        if let Some(top) = self.stack.last_mut() {
                            *top = ContainerState::ObjectExpectColon;
                        }
                    } else {
                        self.finish_value_on_stack();
                    }
                } else if ch == '"' {
                    self.push_str_out("\\\"");
                } else if ch == '\n' {
                    self.push_str_out("\\n");
                } else if ch == '\r' {
                    self.push_str_out("\\r");
                } else if ch == '\t' {
                    self.push_str_out("\\t");
                } else {
                    self.push_out(ch);
                }
            }
            LexicalState::InSingleQuoteEscape { is_key } => {
                match ch {
                    '\'' => self.push_out('\''),
                    '"' => self.push_str_out("\\\""),
                    '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't' => {
                        self.push_out('\\');
                        self.push_out(ch);
                    }
                    _ => self.push_out(ch),
                }
                self.lex = LexicalState::InSingleQuoteString { is_key };
            }
            LexicalState::InNumber {
                saw_digit_after_marker,
            } => {
                if ch.is_ascii_digit() {
                    self.push_out(ch);
                    self.lex = LexicalState::InNumber {
                        saw_digit_after_marker: true,
                    };
                } else if matches!(ch, '.' | 'e' | 'E' | '+' | '-') {
                    self.push_out(ch);
                    self.lex = LexicalState::InNumber {
                        saw_digit_after_marker: false,
                    };
                } else {
                    if !saw_digit_after_marker {
                        self.push_out('0');
                    }
                    self.lex = LexicalState::Normal;
                    self.finish_value_on_stack();
                    self.step(ch);
                }
            }
            LexicalState::InIdentifier { first_char, len } => {
                if ch.is_ascii_alphanumeric() || ch == '_' {
                    self.lex = LexicalState::InIdentifier {
                        first_char,
                        len: len + 1,
                    };
                } else {
                    self.emit_normalized_identifier(first_char);
                    self.lex = LexicalState::Normal;
                    self.finish_value_on_stack();
                    self.step(ch);
                }
            }
            LexicalState::InUnquotedKey => {
                if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '$' {
                    self.push_out(ch);
                } else {
                    self.push_out('"');
                    if let Some(top) = self.stack.last_mut() {
                        *top = ContainerState::ObjectExpectColon;
                    }
                    self.lex = LexicalState::Normal;
                    self.step(ch);
                }
            }
            LexicalState::Normal => self.step_normal(ch),
        }
    }

    fn emit_normalized_identifier(&mut self, first_char: char) {
        let literal = match first_char {
            't' | 'T' => "true",
            'f' | 'F' => "false",
            _ => "null",
        };
        self.push_str_out(literal);
    }

    fn step_normal(&mut self, ch: char) {
        if ch.is_whitespace() {
            return;
        }

        let top_state = self.stack.last().copied();

        match ch {
            '{' => {
                self.prepare_for_value_start();
                self.saw_root_token = true;
                self.committed.push('{');
                self.stack.push(ContainerState::ObjectExpectKeyOrEnd);
            }
            '[' => {
                self.prepare_for_value_start();
                self.saw_root_token = true;
                self.committed.push('[');
                self.stack.push(ContainerState::ArrayExpectValueOrEnd);
            }
            '}' => {
                if matches!(
                    top_state,
                    Some(
                        ContainerState::ObjectExpectKeyOrEnd
                            | ContainerState::ObjectAfterValue
                            | ContainerState::ObjectExpectValue
                            | ContainerState::ObjectExpectColon
                    )
                ) {
                    // Drop any incomplete trailing `, "danglingKey":` cleanly
                    self.pending_member_prefix.clear();
                    self.in_pending_member = false;
                    self.stack.pop();
                    self.committed.push('}');
                    self.finish_value_on_stack();
                }
            }
            ']' => {
                if matches!(
                    top_state,
                    Some(ContainerState::ArrayExpectValueOrEnd | ContainerState::ArrayAfterValue)
                ) {
                    self.stack.pop();
                    self.committed.push(']');
                    self.finish_value_on_stack();
                }
            }
            ',' => match top_state {
                Some(ContainerState::ObjectAfterValue) => {
                    if let Some(top) = self.stack.last_mut() {
                        *top = ContainerState::ObjectExpectKeyOrEnd;
                    }
                    self.in_pending_member = true;
                    self.pending_member_prefix.clear();
                    self.pending_member_prefix.push(',');
                }
                Some(ContainerState::ArrayAfterValue) => {
                    if let Some(top) = self.stack.last_mut() {
                        *top = ContainerState::ArrayExpectValueOrEnd;
                    }
                    self.in_pending_member = true;
                    self.pending_member_prefix.clear();
                    self.pending_member_prefix.push(',');
                }
                _ => {}
            },
            ':' => {
                if matches!(top_state, Some(ContainerState::ObjectExpectColon)) {
                    self.push_out(':');
                    if let Some(top) = self.stack.last_mut() {
                        *top = ContainerState::ObjectExpectValue;
                    }
                }
            }
            '"' | '\'' => {
                let single = ch == '\'';
                if matches!(top_state, Some(ContainerState::ObjectExpectKeyOrEnd)) {
                    if !self.in_pending_member {
                        self.in_pending_member = true;
                        self.pending_member_prefix.clear();
                    }
                    self.push_out('"');
                    self.lex = if single {
                        LexicalState::InSingleQuoteString { is_key: true }
                    } else {
                        LexicalState::InString { is_key: true }
                    };
                } else {
                    self.prepare_for_value_start();
                    self.saw_root_token = true;
                    self.value_string_start_idx = Some(self.committed.len() + 1);
                    self.active_value_string_raw.clear();
                    self.committed.push('"');
                    self.lex = if single {
                        LexicalState::InSingleQuoteString { is_key: false }
                    } else {
                        LexicalState::InString { is_key: false }
                    };
                }
            }
            '-' | '0'..='9' => {
                if matches!(top_state, Some(ContainerState::ObjectExpectKeyOrEnd)) {
                    return;
                }
                self.prepare_for_value_start();
                self.saw_root_token = true;
                self.committed.push(ch);
                self.lex = LexicalState::InNumber {
                    saw_digit_after_marker: ch.is_ascii_digit(),
                };
            }
            c if c.is_ascii_alphabetic() || c == '_' || c == '$' => {
                if matches!(top_state, Some(ContainerState::ObjectExpectKeyOrEnd)) {
                    if !self.in_pending_member {
                        self.in_pending_member = true;
                        self.pending_member_prefix.clear();
                    }
                    self.push_out('"');
                    self.push_out(c);
                    self.lex = LexicalState::InUnquotedKey;
                    return;
                }
                self.prepare_for_value_start();
                self.saw_root_token = true;
                self.lex = LexicalState::InIdentifier {
                    first_char: c,
                    len: 1,
                };
            }
            _ => {}
        }
    }

    #[inline]
    fn prepare_for_value_start(&mut self) {
        if let Some(ContainerState::ObjectExpectColon) = self.stack.last() {
            self.push_out(':');
            if let Some(top) = self.stack.last_mut() {
                *top = ContainerState::ObjectExpectValue;
            }
        }
        // Handle missing comma between consecutive array/object values if an LLM drops a comma
        if let Some(ContainerState::ArrayAfterValue) = self.stack.last() {
            self.committed.push(',');
            if let Some(top) = self.stack.last_mut() {
                *top = ContainerState::ArrayExpectValueOrEnd;
            }
        }
        self.flush_pending_member();
    }

    /// Checks whether a value string contains an escaped JSON object/array (e.g. tool call args)
    /// and repairs it in place if truncated or malformed.
    fn maybe_heal_completed_escaped_json(&mut self) {
        let Some(start_idx) = self.value_string_start_idx else {
            return;
        };
        if !looks_like_escaped_json(&self.active_value_string_raw) {
            return;
        }
        if let Some(repaired_escaped) = heal_escaped_json_inner(&self.active_value_string_raw) {
            self.committed.truncate(start_idx);
            self.committed.push_str(&repaired_escaped);
        }
    }

    /// Synthesizes a 100% syntactically valid JSON snapshot in O(d) time without mutating
    /// the ongoing incremental stream state!
    pub fn snapshot_healed(&self) -> String {
        if !self.saw_root_token && !self.in_pending_member {
            return String::new();
        }

        let mut out = self.committed.clone();

        // Notice: if `in_pending_member` is true, we are inside a trailing `, "key"` or `{"key": `
        // where NO value token has started yet! Dropping `pending_member_prefix` automatically
        // strips dangling incomplete object keys & trailing commas in O(1) time!
        if !self.in_pending_member {
            match self.lex {
                LexicalState::Normal => {}
                LexicalState::InString { is_key: false } => {
                    if self.repair_nested_escaped_json
                        && looks_like_escaped_json(&self.active_value_string_raw)
                    {
                        if let (Some(start_idx), Some(repaired_escaped)) = (
                            self.value_string_start_idx,
                            heal_escaped_json_inner(&self.active_value_string_raw),
                        ) {
                            out.truncate(start_idx);
                            out.push_str(&repaired_escaped);
                        }
                    }
                    out.push('"');
                }
                LexicalState::InStringEscape { is_key: false } => {
                    // Strip dangling trailing backslash before closing quote
                    out.pop();
                    if self.repair_nested_escaped_json {
                        let mut trimmed_raw = self.active_value_string_raw.clone();
                        trimmed_raw.pop();
                        if looks_like_escaped_json(&trimmed_raw) {
                            if let (Some(start_idx), Some(repaired_escaped)) = (
                                self.value_string_start_idx,
                                heal_escaped_json_inner(&trimmed_raw),
                            ) {
                                out.truncate(start_idx);
                                out.push_str(&repaired_escaped);
                            }
                        }
                    }
                    out.push('"');
                }
                LexicalState::InUnicodeEscape {
                    is_key: false,
                    remaining_hex,
                } => {
                    for _ in 0..remaining_hex {
                        out.push('0');
                    }
                    out.push('"');
                }
                LexicalState::InSingleQuoteString { is_key: false }
                | LexicalState::InSingleQuoteEscape { is_key: false } => {
                    out.push('"');
                }
                LexicalState::InNumber {
                    saw_digit_after_marker,
                } => {
                    if !saw_digit_after_marker {
                        out.push('0');
                    }
                }
                LexicalState::InIdentifier { first_char, .. } => {
                    let lit = match first_char {
                        't' | 'T' => "true",
                        'f' | 'F' => "false",
                        _ => "null",
                    };
                    out.push_str(lit);
                }
                _ => {}
            }
        }

        // Close all active containers on the pushdown stack in LIFO order: O(d)
        for state in self.stack.iter().rev() {
            match state {
                ContainerState::ObjectExpectKeyOrEnd
                | ContainerState::ObjectExpectColon
                | ContainerState::ObjectExpectValue
                | ContainerState::ObjectAfterValue => out.push('}'),
                ContainerState::ArrayExpectValueOrEnd | ContainerState::ArrayAfterValue => {
                    out.push(']')
                }
            }
        }

        if out.is_empty() && self.in_pending_member {
            return "{}".to_string();
        }
        out
    }
}

#[inline]
fn looks_like_escaped_json(raw_escaped: &str) -> bool {
    let trimmed = raw_escaped.trim_start();
    (trimmed.starts_with('{') || trimmed.starts_with('[')) && trimmed.contains("\\\"")
}

fn heal_escaped_json_inner(raw_escaped: &str) -> Option<String> {
    // Unescape one level (`\"` -> `"`, `\\` -> `\`, `\n` -> newline)
    let mut unescaped = String::with_capacity(raw_escaped.len());
    let mut chars = raw_escaped.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('"') => unescaped.push('"'),
                Some('\\') => unescaped.push('\\'),
                Some('n') => unescaped.push('\n'),
                Some('r') => unescaped.push('\r'),
                Some('t') => unescaped.push('\t'),
                Some('/') => unescaped.push('/'),
                Some(other) => {
                    unescaped.push('\\');
                    unescaped.push(other);
                }
                None => {}
            }
        } else {
            unescaped.push(c);
        }
    }

    let mut inner_pda = IncrementalJsonPda::new(false);
    inner_pda.push_chunk(&unescaped);
    let healed_inner = inner_pda.snapshot_healed();
    if healed_inner.is_empty() {
        return None;
    }

    // Re-escape back into a valid JSON string body
    let mut reescaped = String::with_capacity(healed_inner.len() + 8);
    for c in healed_inner.chars() {
        match c {
            '"' => reescaped.push_str("\\\""),
            '\\' => reescaped.push_str("\\\\"),
            '\n' => reescaped.push_str("\\n"),
            '\r' => reescaped.push_str("\\r"),
            '\t' => reescaped.push_str("\\t"),
            _ => reescaped.push(c),
        }
    }
    Some(reescaped)
}

/// Convenience stateless one-shot JSON healer using the incremental PDA.
pub fn heal_json(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let mut pda = IncrementalJsonPda::new(true);
    pda.push_chunk(trimmed);
    pda.snapshot_healed()
}
