// Shared tree-sitter-backed HDL parsing helpers for hdl-tools nodes.
//
// The generated service.rs only wires the node files listed in axiom.yaml as
// crate modules, so shared code cannot live at the crate root. Each node that
// needs these helpers includes this file as its own submodule with
// `#[path = "hdlutil.rs"] mod hdlutil;` (the `#[path]` is resolved relative to
// nodes/). It is compiled once per including node; every function is pure.
//
// Two grammars do the actual parsing:
//   - tree_sitter_systemverilog — IEEE 1800-2017 SystemVerilog (Verilog is a
//     strict subset, so this one grammar covers both dialects).
//   - tree_sitter_vhdl — VHDL, parsing entity/port/generic declarations
//     directly (not only the legacy component-mirror idiom older tools need).
// This module's own code only WALKS the resulting syntax tree to pull out
// interface-level facts; it never re-implements HDL grammar or semantics.
#![allow(dead_code)]

use crate::gen::messages::{
    HdlModule, HdlPackage, HdlPackageItem, HdlParam, HdlPort, HdlSyntaxIssue,
};
use tree_sitter::{Node, Parser, Tree};

/// Max accepted source length in bytes, checked on the RAW input before any
/// parse. Bounds allocation and output size up front. (~2 MiB, matching
/// christiangeorgelucas/code-parse-tools' cap for the same reason.)
pub const MAX_SOURCE_BYTES: usize = 2_000_000;
/// Max number of modules/entities/packages collected from one source. Bounds
/// output size on a pathological file with huge numbers of tiny definitions.
pub const MAX_DEFINITIONS: usize = 5_000;
/// Max number of ports or parameters/generics collected per module/entity.
pub const MAX_ITEMS_PER_DEFINITION: usize = 5_000;
/// Max number of package items (types/subtypes/constants/functions/
/// procedures) collected per VHDL package.
pub const MAX_PACKAGE_ITEMS: usize = 5_000;
/// Max number of syntax-error/missing regions collected by ValidateSyntax.
pub const MAX_ISSUES: usize = 20_000;
/// Max bytes of source text copied into a single text field (type/value/doc
/// slices). Keeps a single response bounded well under the platform's 4 MiB
/// gRPC message limit even on a pathological single-line declaration.
pub const MAX_SLICE_BYTES: usize = 4_096;
/// Max nodes visited by an iterative whole-tree scan (definitions, comments,
/// error collection). Bounds work on a huge, mostly-flat file without
/// touching recursion at all (every walk in this module is iterative).
pub const MAX_SCAN_NODES: usize = 2_000_000;

/// Enforce the raw-input size cap. Returns "INPUT_TOO_LARGE" if exceeded.
pub fn guard_source(text: &str) -> Result<(), String> {
    if text.len() > MAX_SOURCE_BYTES {
        return Err("INPUT_TOO_LARGE".to_string());
    }
    Ok(())
}

/// Parse `text` with the SystemVerilog grammar (covers Verilog too).
pub fn parse_verilog(text: &str) -> Result<Tree, String> {
    guard_source(text)?;
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_systemverilog::LANGUAGE.into())
        .map_err(|_| "PARSE_FAILED".to_string())?;
    parser.parse(text, None).ok_or_else(|| "PARSE_FAILED".to_string())
}

/// Parse `text` with the VHDL grammar.
pub fn parse_vhdl(text: &str) -> Result<Tree, String> {
    guard_source(text)?;
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_vhdl::LANGUAGE.into())
        .map_err(|_| "PARSE_FAILED".to_string())?;
    parser.parse(text, None).ok_or_else(|| "PARSE_FAILED".to_string())
}

/// Normalize a language hint (case-insensitive, trimmed) to "verilog",
/// "vhdl", or "" for an empty/unrecognized hint (auto-detect).
pub fn normalize_language_hint(hint: &str) -> &'static str {
    match hint.trim().to_ascii_lowercase().as_str() {
        "verilog" | "systemverilog" | "sv" | "v" => "verilog",
        "vhdl" | "vhd" => "vhdl",
        _ => "",
    }
}

/// Result of auto-detecting Verilog/SystemVerilog vs VHDL by parsing with
/// both grammars and comparing syntax-error counts.
pub struct DetectResult {
    pub language: String,
    pub confidence: f64,
    pub verilog_errors: u32,
    pub vhdl_errors: u32,
}

/// Parse `text` with BOTH grammars and pick whichever parsed with fewer
/// ERROR/MISSING nodes — an objective, parser-verified signal rather than a
/// keyword guess. "" language means the two grammars gave no useful signal
/// (either both parsed perfectly cleanly — e.g. trivial/near-empty input —
/// or tied with the same nonzero error count).
pub fn detect_language(text: &str) -> Result<DetectResult, String> {
    guard_source(text)?;
    let v_tree = parse_verilog(text)?;
    let d_tree = parse_vhdl(text)?;
    let v_err = count_error_nodes(&v_tree);
    let d_err = count_error_nodes(&d_tree);
    let (language, confidence) = if v_err == 0 && d_err == 0 {
        (String::new(), 0.0)
    } else if v_err == d_err {
        (String::new(), 0.5)
    } else if v_err < d_err {
        ("verilog".to_string(), 1.0 - (v_err as f64 / (v_err + d_err) as f64))
    } else {
        ("vhdl".to_string(), 1.0 - (d_err as f64 / (v_err + d_err) as f64))
    };
    Ok(DetectResult { language, confidence, verilog_errors: v_err, vhdl_errors: d_err })
}

/// Resolve which language a "hint-or-auto-detect" node should use: an
/// explicit, recognized hint always wins (skips the double-parse); otherwise
/// falls back to full detection. Returns "verilog", "vhdl", or "" (detection
/// was ambiguous).
pub fn resolve_language(text: &str, hint: &str) -> Result<String, String> {
    let normalized = normalize_language_hint(hint);
    if !normalized.is_empty() {
        guard_source(text)?;
        return Ok(normalized.to_string());
    }
    Ok(detect_language(text)?.language)
}

/// A UTF-8 safe, byte-bounded slice of `source` for a node span.
pub fn slice(source: &str, start: usize, end: usize) -> String {
    let end = end.min(source.len());
    if start >= end {
        return String::new();
    }
    let raw = &source.as_bytes()[start..end];
    if raw.len() <= MAX_SLICE_BYTES {
        String::from_utf8_lossy(raw).into_owned()
    } else {
        let mut cut = MAX_SLICE_BYTES;
        while cut > 0 && !source.is_char_boundary(start + cut) {
            cut -= 1;
        }
        let mut s = String::from_utf8_lossy(&raw[..cut]).into_owned();
        s.push_str("...");
        s
    }
}

fn node_text(node: Node, source: &str) -> String {
    slice(source, node.start_byte(), node.end_byte())
}

/// Count ERROR + MISSING nodes in a tree via an iterative DFS (no recursion —
/// safe on deeply nested/adversarial input). Bounded by MAX_SCAN_NODES.
pub fn count_error_nodes(tree: &Tree) -> u32 {
    if !tree.root_node().has_error() {
        return 0;
    }
    let mut count: u32 = 0;
    let mut cursor = tree.walk();
    let mut visited: usize = 0;
    loop {
        let node = cursor.node();
        visited += 1;
        if visited > MAX_SCAN_NODES {
            return count;
        }
        if node.is_error() || node.is_missing() {
            count += 1;
        }
        if cursor.goto_first_child() {
            continue;
        }
        loop {
            if cursor.goto_next_sibling() {
                break;
            }
            if !cursor.goto_parent() {
                return count;
            }
        }
    }
}

/// Collect every ERROR/MISSING region in a tree (iterative DFS, bounded by
/// MAX_ISSUES and MAX_SCAN_NODES).
pub fn collect_issues(tree: &Tree) -> Vec<HdlSyntaxIssue> {
    let mut out = Vec::new();
    if !tree.root_node().has_error() {
        return out;
    }
    let mut cursor = tree.walk();
    let mut visited: usize = 0;
    loop {
        let node = cursor.node();
        visited += 1;
        if out.len() >= MAX_ISSUES || visited > MAX_SCAN_NODES {
            return out;
        }
        if node.is_error() || node.is_missing() {
            let sp = node.start_position();
            let ep = node.end_position();
            out.push(HdlSyntaxIssue {
                kind: if node.is_missing() { "MISSING" } else { "ERROR" }.to_string(),
                start_byte: node.start_byte() as u32,
                end_byte: node.end_byte() as u32,
                start_row: sp.row as u32,
                start_column: sp.column as u32,
                end_row: ep.row as u32,
                end_column: ep.column as u32,
            });
        }
        if cursor.goto_first_child() {
            continue;
        }
        loop {
            if cursor.goto_next_sibling() {
                break;
            }
            if !cursor.goto_parent() {
                return out;
            }
        }
    }
}

/// Find every descendant node (anywhere in the tree, iterative DFS, no
/// recursion) whose kind is exactly `kind`. Bounded by MAX_SCAN_NODES visited
/// and `max_results` collected.
fn find_all_of_kind<'a>(root: Node<'a>, kind: &str, max_results: usize) -> Vec<Node<'a>> {
    let mut out = Vec::new();
    let mut cursor = root.walk();
    let mut visited: usize = 0;
    loop {
        let node = cursor.node();
        visited += 1;
        if node.kind() == kind {
            out.push(node);
            if out.len() >= max_results {
                return out;
            }
        }
        if visited > MAX_SCAN_NODES {
            return out;
        }
        if cursor.goto_first_child() {
            continue;
        }
        loop {
            if cursor.goto_next_sibling() {
                break;
            }
            if !cursor.goto_parent() {
                return out;
            }
        }
    }
}

/// True if `kind` names a comment node in either grammar.
fn is_comment_kind(kind: &str) -> bool {
    matches!(kind, "one_line_comment" | "block_comment" | "line_comment")
}

/// The human-readable text of a comment node: prefer a nested `comment_content`
/// child (VHDL's line_comment wraps its text this way, already stripped of
/// the leading `--`), else strip the SystemVerilog `//`/`/* */` delimiters
/// ourselves. Always trimmed.
fn comment_text(node: Node, source: &str) -> String {
    for i in 0..node.child_count() {
        if let Some(c) = node.child(i) {
            if c.kind() == "comment_content" {
                return node_text(c, source).trim().to_string();
            }
        }
    }
    let raw = node_text(node, source);
    let raw = raw.trim();
    let raw = raw.strip_prefix("//").unwrap_or(raw);
    let raw = raw.strip_prefix("/*").unwrap_or(raw);
    let raw = raw.strip_suffix("*/").unwrap_or(raw);
    let raw = raw.strip_prefix("--").unwrap_or(raw);
    raw.trim().to_string()
}

/// A comment node's "effective" end row for line-adjacency comparisons.
/// VHDL's `line_comment` includes its trailing newline in its own byte range
/// (so `end_position().row` already lands on the FOLLOWING source line);
/// SystemVerilog's `one_line_comment` does not. Normalize both to "the row
/// the comment's visible text is actually on" by checking the raw byte
/// immediately before the node's end.
fn effective_comment_end_row(node: Node, source: &str) -> u32 {
    let row = node.end_position().row as u32;
    let end = node.end_byte();
    if end > 0 && source.as_bytes().get(end - 1) == Some(&b'\n') {
        row.saturating_sub(1)
    } else {
        row
    }
}

/// The doc comment immediately preceding `target` among its own siblings (a
/// comment ending on the line directly above target's start line, with no
/// other non-comment sibling in between). Used for module/entity/package
/// level "leading" doc comments. Empty if none.
pub fn leading_doc(target: Node, source: &str) -> String {
    let Some(parent) = target.parent() else { return String::new() };
    let mut prev: Option<Node> = None;
    for i in 0..parent.child_count() {
        if let Some(c) = parent.child(i) {
            if c.id() == target.id() {
                break;
            }
            prev = Some(c);
        }
    }
    match prev {
        Some(c) if is_comment_kind(c.kind()) => {
            if effective_comment_end_row(c, source) + 1 == target.start_position().row as u32 {
                comment_text(c, source)
            } else {
                String::new()
            }
        }
        _ => String::new(),
    }
}

/// The doc comment associated with one item inside a sibling LIST (a port
/// within a port list, a generic within a generic list): prefer a trailing
/// comment on the same source line immediately after the item (before the
/// next item), else a leading comment on the line directly above — but ONLY
/// if that comment is not already the trailing comment of the item two
/// positions back (otherwise a single trailing "// foo" comment after item A
/// would be double-attributed: once as A's trailing doc, again as the NEXT
/// item B's leading doc, which is wrong — it describes A, not B).
pub fn item_doc(item: Node, source: &str) -> String {
    let Some(parent) = item.parent() else { return String::new() };
    let mut children: Vec<Node> = Vec::with_capacity(parent.child_count());
    for i in 0..parent.child_count() {
        if let Some(c) = parent.child(i) {
            children.push(c);
        }
    }
    let Some(idx) = children.iter().position(|c| c.id() == item.id()) else {
        return String::new();
    };

    // Trailing: nearest following named sibling (skipping anonymous
    // punctuation), if it's a comment on the same source line as item's end.
    let mut j = idx + 1;
    while j < children.len() && !children[j].is_named() {
        j += 1;
    }
    if j < children.len() {
        let cand = children[j];
        if is_comment_kind(cand.kind()) && cand.start_position().row == item.end_position().row {
            return comment_text(cand, source);
        }
    }

    // Leading: nearest preceding named sibling (skipping anonymous
    // punctuation), if it's a comment ending the line directly above item.
    let mut k = idx;
    let mut prev_named_idx: Option<usize> = None;
    while k > 0 {
        k -= 1;
        if children[k].is_named() {
            prev_named_idx = Some(k);
            break;
        }
    }
    let Some(k) = prev_named_idx else { return String::new() };
    let cand = children[k];
    if !is_comment_kind(cand.kind())
        || effective_comment_end_row(cand, source) + 1 != item.start_position().row as u32
    {
        return String::new();
    }
    // Guard against double-attribution: if `cand` is itself the trailing
    // comment of the NEXT sibling back (same row as that sibling's end),
    // it already belongs to that earlier item — do not also give it to us.
    let mut m = k;
    let mut earlier_named: Option<Node> = None;
    while m > 0 {
        m -= 1;
        if children[m].is_named() {
            earlier_named = Some(children[m]);
            break;
        }
    }
    if let Some(earlier) = earlier_named {
        if cand.start_position().row == earlier.end_position().row {
            return String::new();
        }
    }
    comment_text(cand, source)
}

// ─── Verilog / SystemVerilog ────────────────────────────────────────────────

/// One decoded Verilog/SystemVerilog port or parameter before it is packaged
/// into the output message shape the caller wants.
struct RawItem {
    name: String,
    mode: String, // direction (ports) — empty for parameters
    data_type: String,
    width: String, // ports only
    default_value: String, // parameters only
    doc: String,
}

/// Find the first descendant of `root` (root included) whose kind is `kind`,
/// via a bounded iterative DFS. Used only on tiny, fixed-shape subtrees (a
/// single port/type header), so a small fixed cap is enough — this is not on
/// the whole-file scan path (that's `find_all_of_kind`, capped separately).
fn find_first_descendant_of_kind<'a>(root: Node<'a>, kind: &str) -> Option<Node<'a>> {
    let mut stack = vec![root];
    let mut visited = 0usize;
    while let Some(n) = stack.pop() {
        visited += 1;
        if visited > 2_000 {
            return None;
        }
        if n.kind() == kind {
            return Some(n);
        }
        for i in (0..n.child_count()).rev() {
            if let Some(c) = n.child(i) {
                stack.push(c);
            }
        }
    }
    None
}

/// Split a Verilog port-type container (net_port_type / variable_port_type)
/// into (base type text, width text). The width is the first packed/unpacked
/// dimension found ANYWHERE inside the container (it can be nested a few
/// levels down, e.g. variable_port_type > data_type > packed_dimension for a
/// `reg`/`logic` port) — everything else is the base type, with the width
/// substring excised and whitespace collapsed.
fn split_verilog_type_width(container: Node, source: &str) -> (String, String) {
    let dim = find_first_descendant_of_kind(container, "packed_dimension")
        .or_else(|| find_first_descendant_of_kind(container, "unpacked_dimension"));
    match dim {
        Some(d) => {
            let width = node_text(d, source);
            let before = slice(source, container.start_byte(), d.start_byte());
            let after = slice(source, d.end_byte(), container.end_byte());
            let base = format!("{} {}", before, after).split_whitespace().collect::<Vec<_>>().join(" ");
            (base, width)
        }
        None => {
            let base = node_text(container, source).split_whitespace().collect::<Vec<_>>().join(" ");
            (base, String::new())
        }
    }
}

/// Decode one `ansi_port_declaration` node into a RawItem.
fn decode_ansi_port(decl: Node, source: &str) -> RawItem {
    let name = decl
        .child_by_field_name("port_name")
        .map(|n| node_text(n, source))
        .unwrap_or_default();
    let mut mode = String::new();
    let mut data_type = String::new();
    let mut width = String::new();
    for i in 0..decl.child_count() {
        let Some(c) = decl.child(i) else { continue };
        match c.kind() {
            "net_port_header" | "variable_port_header" | "interface_port_header" => {
                for j in 0..c.child_count() {
                    let Some(gc) = c.child(j) else { continue };
                    match gc.kind() {
                        "port_direction" => mode = node_text(gc, source),
                        "net_port_type" | "variable_port_type" => {
                            let (t, w) = split_verilog_type_width(gc, source);
                            data_type = t;
                            width = w;
                        }
                        _ => {}
                    }
                }
            }
            "port_direction" => mode = node_text(c, source),
            _ => {}
        }
    }
    RawItem { name, mode, data_type, width, default_value: String::new(), doc: item_doc(decl, source) }
}

/// Decode a non-ANSI module's body-level `input_declaration` /
/// `output_declaration` / `inout_declaration` into (direction, base type,
/// width, names). One declaration can list several names sharing the mode.
fn decode_nonansi_port_decl(decl: Node, source: &str) -> (String, String, String, Vec<String>) {
    let mode = match decl.kind() {
        "input_declaration" => "input",
        "output_declaration" => "output",
        "inout_declaration" => "inout",
        _ => "",
    }
    .to_string();
    let mut data_type = String::new();
    let mut width = String::new();
    let mut names = Vec::new();
    for i in 0..decl.child_count() {
        let Some(c) = decl.child(i) else { continue };
        match c.kind() {
            "net_port_type" | "variable_port_type" => {
                let (t, w) = split_verilog_type_width(c, source);
                data_type = t;
                width = w;
            }
            "list_of_port_identifiers" | "list_of_variable_identifiers"
            | "list_of_variable_port_identifiers" => {
                for j in 0..c.child_count() {
                    let Some(gc) = c.child(j) else { continue };
                    if gc.kind() == "simple_identifier" || gc.kind() == "escaped_identifier" {
                        names.push(node_text(gc, source));
                    }
                }
            }
            _ => {}
        }
    }
    (mode, data_type, width, names)
}

/// Decode one `param_assignment` (inside a `parameter_declaration`) into a
/// RawItem, given the declaration's shared type text (may be empty).
fn decode_param_assignment(assign: Node, shared_type: &str, source: &str) -> RawItem {
    let mut name = String::new();
    let mut default_value = String::new();
    let mut seen_eq = false;
    for i in 0..assign.child_count() {
        let Some(c) = assign.child(i) else { continue };
        if c.kind() == "simple_identifier" || c.kind() == "escaped_identifier" {
            if name.is_empty() {
                name = node_text(c, source);
            }
        } else if !c.is_named() && node_text(c, source) == "=" {
            seen_eq = true;
        } else if seen_eq && default_value.is_empty() && c.is_named() {
            default_value = node_text(c, source);
        }
    }
    RawItem {
        name,
        mode: String::new(),
        data_type: shared_type.to_string(),
        width: String::new(),
        default_value,
        doc: item_doc(assign, source),
    }
}

/// Extract every `parameter` from a `parameter_port_list` (the ANSI `#(...)`
/// header block). Ignores `localparam`s — they cannot be overridden at
/// instantiation, so they are not part of the externally-visible interface.
fn extract_verilog_parameters(header: Node, source: &str) -> Vec<HdlParam> {
    let mut out = Vec::new();
    let Some(param_list) = find_child_kind(header, "parameter_port_list") else { return out };
    for decl in find_all_of_kind(param_list, "parameter_declaration", MAX_ITEMS_PER_DEFINITION) {
        let mut shared_type = String::new();
        for i in 0..decl.child_count() {
            let Some(c) = decl.child(i) else { continue };
            if c.kind() == "data_type_or_implicit" {
                shared_type = node_text(c, source);
            }
        }
        for i in 0..decl.child_count() {
            let Some(c) = decl.child(i) else { continue };
            if c.kind() == "list_of_param_assignments" {
                for j in 0..c.child_count() {
                    let Some(pa) = c.child(j) else { continue };
                    if pa.kind() == "param_assignment" {
                        if out.len() >= MAX_ITEMS_PER_DEFINITION {
                            return out;
                        }
                        let raw = decode_param_assignment(pa, &shared_type, source);
                        out.push(HdlParam {
                            name: raw.name,
                            data_type: raw.data_type,
                            default_value: raw.default_value,
                            doc: raw.doc,
                        });
                    }
                }
            }
        }
    }
    out
}

fn find_child_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    for i in 0..node.child_count() {
        if let Some(c) = node.child(i) {
            if c.kind() == kind {
                return Some(c);
            }
        }
    }
    None
}

/// Extract the full interface (name, ports, parameters, doc) of one
/// `module_declaration` node. Handles both ANSI (`module foo #(...)(...)`)
/// and classic non-ANSI (`module foo(a,b); input a; ...`) header styles.
fn extract_verilog_module(module_decl: Node, source: &str) -> HdlModule {
    let (header, is_ansi) = match find_child_kind(module_decl, "module_ansi_header") {
        Some(h) => (h, true),
        None => match find_child_kind(module_decl, "module_nonansi_header") {
            Some(h) => (h, false),
            None => return HdlModule { kind: "module".to_string(), ..Default::default() },
        },
    };
    let name = header
        .child_by_field_name("name")
        .map(|n| node_text(n, source))
        .unwrap_or_default();
    let parameters = extract_verilog_parameters(header, source);

    let mut ports = Vec::new();
    if is_ansi {
        if let Some(port_list) = find_child_kind(header, "list_of_port_declarations") {
            for decl in find_all_of_kind(port_list, "ansi_port_declaration", MAX_ITEMS_PER_DEFINITION) {
                let raw = decode_ansi_port(decl, source);
                if raw.name.is_empty() {
                    continue;
                }
                ports.push(HdlPort {
                    name: raw.name,
                    direction: raw.mode,
                    data_type: raw.data_type,
                    width: raw.width,
                    doc: raw.doc,
                });
            }
        }
    } else {
        // Ordered port names from the non-ANSI header's parenthesized list.
        let mut ordered_names: Vec<String> = Vec::new();
        if let Some(list) = find_child_kind(header, "list_of_ports") {
            for p in find_all_of_kind(list, "port", MAX_ITEMS_PER_DEFINITION) {
                for i in 0..p.child_count() {
                    if let Some(c) = p.child(i) {
                        if c.kind() == "simple_identifier" || c.kind() == "escaped_identifier" {
                            ordered_names.push(node_text(c, source));
                            break;
                        }
                    }
                }
            }
        }
        // Body-level input/output/inout declarations, keyed by name.
        let mut resolved: std::collections::HashMap<String, (String, String, String, String)> =
            std::collections::HashMap::new();
        for kind in ["input_declaration", "output_declaration", "inout_declaration"] {
            for decl in find_all_of_kind(module_decl, kind, MAX_ITEMS_PER_DEFINITION) {
                let (mode, data_type, width, names) = decode_nonansi_port_decl(decl, source);
                let doc = item_doc(decl, source);
                for n in names {
                    resolved.insert(n, (mode.clone(), data_type.clone(), width.clone(), doc.clone()));
                }
            }
        }
        for n in ordered_names {
            if ports.len() >= MAX_ITEMS_PER_DEFINITION {
                break;
            }
            let (mode, data_type, width, doc) =
                resolved.get(&n).cloned().unwrap_or_default();
            ports.push(HdlPort { name: n, direction: mode, data_type, width, doc });
        }
    }

    HdlModule {
        name,
        kind: "module".to_string(),
        ports,
        parameters,
        doc: leading_doc(module_decl, source),
        // has_error on the module's OWN subtree (not the whole file): a
        // syntax problem elsewhere in the source must not taint an otherwise
        // clean module, but a problem inside THIS module's header/port list
        // (e.g. a missing separator between two ports) can silently produce
        // wrong ports/directions above, so callers need this signal.
        has_error: module_decl.has_error(),
    }
}

/// Find every module defined anywhere in a parsed Verilog/SystemVerilog tree.
pub fn find_all_verilog_modules(tree: &Tree, source: &str) -> Vec<HdlModule> {
    find_all_of_kind(tree.root_node(), "module_declaration", MAX_DEFINITIONS)
        .into_iter()
        .map(|n| extract_verilog_module(n, source))
        .collect()
}

// ─── VHDL ───────────────────────────────────────────────────────────────────

/// Split a VHDL `subtype_indication`'s `type` field node into (base type
/// text, width/range text). The width is the raw text of a nested
/// `parenthesis_group`, if any (e.g. "(7 downto 0)").
fn split_vhdl_type_width(type_node: Node, source: &str) -> (String, String) {
    let mut width = String::new();
    let mut base_end = type_node.end_byte();
    for i in 0..type_node.child_count() {
        if let Some(c) = type_node.child(i) {
            if c.kind() == "parenthesis_group" {
                width = node_text(c, source);
                base_end = c.start_byte();
                break;
            }
        }
    }
    (slice(source, type_node.start_byte(), base_end).trim().to_string(), width)
}

/// Decode one `interface_declaration` (or the `interface_signal_declaration` /
/// `interface_constant_declaration` variants used in subprogram parameter
/// lists) into zero or more RawItems — VHDL allows `a, b, c : in std_logic`
/// to declare several names sharing one mode/type/default.
fn decode_vhdl_interface_decl(decl: Node, source: &str) -> Vec<RawItem> {
    let mut names = Vec::new();
    let mut mode = String::new();
    let mut data_type = String::new();
    let mut width = String::new();
    let mut default_value = String::new();
    for i in 0..decl.child_count() {
        let Some(c) = decl.child(i) else { continue };
        match c.kind() {
            "identifier_list" => {
                for j in 0..c.child_count() {
                    if let Some(gc) = c.child(j) {
                        // identifier_list's only named children are the names
                        // themselves (comma-separated) — but the grammar's
                        // ambiguity resolution sometimes tags one "identifier"
                        // and another "library_type" for the very same
                        // syntactic position (e.g. a generic later used in an
                        // expression like WIDTH-1 can parse either way), so
                        // match on is_named() rather than a fixed kind list.
                        if gc.is_named() {
                            names.push(node_text(gc, source));
                        }
                    }
                }
            }
            "simple_mode_indication" => {
                for j in 0..c.child_count() {
                    let Some(gc) = c.child(j) else { continue };
                    match gc.kind() {
                        "mode" => mode = node_text(gc, source),
                        "subtype_indication" => {
                            if let Some(t) = gc.child_by_field_name("type") {
                                let (bt, w) = split_vhdl_type_width(t, source);
                                data_type = bt;
                                width = w;
                            }
                        }
                        "initialiser" => {
                            // "initialiser" = ":=" token + the value expression.
                            for k in 0..gc.child_count() {
                                if let Some(v) = gc.child(k) {
                                    if v.is_named() {
                                        default_value = node_text(v, source);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    let doc = item_doc(decl, source);
    names
        .into_iter()
        .map(|n| RawItem {
            name: n,
            mode: mode.clone(),
            data_type: data_type.clone(),
            width: width.clone(),
            default_value: default_value.clone(),
            doc: doc.clone(),
        })
        .collect()
}

/// Normalize a VHDL mode ("in"/"out"/"inout"/"buffer") to our HdlPort
/// direction vocabulary. VHDL generics/ports with no explicit mode default
/// to "in" per the LRM.
fn normalize_vhdl_mode(mode: &str) -> String {
    match mode {
        "in" => "input",
        "out" => "output",
        "inout" => "inout",
        "buffer" => "buffer",
        "" => "input",
        other => other,
    }
    .to_string()
}

/// Decode a `generic_clause` or `port_clause`'s `interface_list` into
/// RawItems, bounded by MAX_ITEMS_PER_DEFINITION.
fn decode_vhdl_interface_list(clause: Node, source: &str) -> Vec<RawItem> {
    let mut out = Vec::new();
    let Some(list) = find_child_kind(clause, "interface_list") else { return out };
    for i in 0..list.child_count() {
        if out.len() >= MAX_ITEMS_PER_DEFINITION {
            break;
        }
        let Some(c) = list.child(i) else { continue };
        if matches!(
            c.kind(),
            "interface_declaration" | "interface_signal_declaration" | "interface_constant_declaration"
        ) {
            out.extend(decode_vhdl_interface_decl(c, source));
        }
    }
    out.truncate(MAX_ITEMS_PER_DEFINITION);
    out
}

/// Extract the full interface (name, ports, generics, doc) of one
/// `entity_declaration` node.
fn extract_vhdl_entity(entity_decl: Node, source: &str) -> HdlModule {
    let name = entity_decl
        .child_by_field_name("entity")
        .map(|n| node_text(n, source))
        .unwrap_or_default();
    let mut ports = Vec::new();
    let mut parameters = Vec::new();
    if let Some(head) = find_child_kind(entity_decl, "entity_head") {
        if let Some(generic_clause) = find_child_kind(head, "generic_clause") {
            for raw in decode_vhdl_interface_list(generic_clause, source) {
                parameters.push(HdlParam {
                    name: raw.name,
                    data_type: raw.data_type,
                    default_value: raw.default_value,
                    doc: raw.doc,
                });
            }
        }
        if let Some(port_clause) = find_child_kind(head, "port_clause") {
            for raw in decode_vhdl_interface_list(port_clause, source) {
                ports.push(HdlPort {
                    name: raw.name,
                    direction: normalize_vhdl_mode(&raw.mode),
                    data_type: raw.data_type,
                    width: raw.width,
                    doc: raw.doc,
                });
            }
        }
    }
    HdlModule {
        name,
        kind: "entity".to_string(),
        ports,
        parameters,
        doc: leading_doc(entity_decl, source),
        has_error: entity_decl.has_error(),
    }
}

/// Find every entity defined anywhere in a parsed VHDL tree.
pub fn find_all_vhdl_entities(tree: &Tree, source: &str) -> Vec<HdlModule> {
    find_all_of_kind(tree.root_node(), "entity_declaration", MAX_DEFINITIONS)
        .into_iter()
        .map(|n| extract_vhdl_entity(n, source))
        .collect()
}

/// Decode a `function_specification` or `procedure_specification` into an
/// HdlPackageItem.
fn decode_vhdl_subprogram(spec: Node, source: &str) -> HdlPackageItem {
    let is_function = spec.kind() == "function_specification";
    let kind = if is_function { "function" } else { "procedure" }.to_string();
    let field = if is_function { "function" } else { "procedure" };
    let name = spec.child_by_field_name(field).map(|n| node_text(n, source)).unwrap_or_default();
    let mut parameters = Vec::new();
    if let Some(param_spec) = find_child_kind(spec, "parameter_list_specification") {
        if let Some(list) = find_child_kind(param_spec, "interface_list") {
            for i in 0..list.child_count() {
                if parameters.len() >= MAX_ITEMS_PER_DEFINITION {
                    break;
                }
                let Some(c) = list.child(i) else { continue };
                if matches!(
                    c.kind(),
                    "interface_declaration" | "interface_signal_declaration" | "interface_constant_declaration"
                ) {
                    for raw in decode_vhdl_interface_decl(c, source) {
                        parameters.push(HdlParam {
                            name: raw.name,
                            data_type: raw.data_type,
                            default_value: raw.default_value,
                            doc: raw.doc,
                        });
                    }
                }
            }
        }
    }
    let return_type = if is_function {
        spec.child_by_field_name("type").map(|n| node_text(n, source)).unwrap_or_default()
    } else {
        String::new()
    };
    HdlPackageItem {
        name,
        kind,
        type_info: String::new(),
        value: String::new(),
        parameters,
        return_type,
        doc: item_doc(spec, source),
    }
}

/// Extract the directly-declared items of one `package_declaration` node:
/// types, subtypes, constants, and function/procedure signatures declared in
/// the package header (not in its body — VHDL separates the two).
fn extract_vhdl_package(pkg_decl: Node, source: &str) -> HdlPackage {
    let name = pkg_decl
        .child_by_field_name("package")
        .map(|n| node_text(n, source))
        .unwrap_or_default();
    let mut items = Vec::new();
    let Some(body) = find_child_kind(pkg_decl, "package_declaration_body") else {
        return HdlPackage { name, items };
    };
    for i in 0..body.child_count() {
        if items.len() >= MAX_PACKAGE_ITEMS {
            break;
        }
        let Some(c) = body.child(i) else { continue };
        match c.kind() {
            "type_declaration" => {
                let item_name =
                    c.child_by_field_name("type").map(|n| node_text(n, source)).unwrap_or_default();
                // The definition node is whatever named child follows the type
                // name and "is" keyword — e.g. enumeration_type_definition,
                // array_type_definition, record_type_definition.
                let mut type_info = String::new();
                for j in 0..c.child_count() {
                    if let Some(gc) = c.child(j) {
                        if gc.is_named() && gc.kind().ends_with("_definition") {
                            type_info = gc
                                .kind()
                                .trim_end_matches("_type_definition")
                                .trim_end_matches("_definition")
                                .to_string();
                            break;
                        }
                    }
                }
                items.push(HdlPackageItem {
                    name: item_name,
                    kind: "type".to_string(),
                    type_info,
                    value: String::new(),
                    parameters: Vec::new(),
                    return_type: String::new(),
                    doc: item_doc(c, source),
                });
            }
            "subtype_declaration" => {
                let item_name =
                    c.child_by_field_name("type").map(|n| node_text(n, source)).unwrap_or_default();
                let mut type_info = String::new();
                if let Some(si) = find_child_kind(c, "subtype_indication") {
                    if let Some(t) = si.child_by_field_name("type") {
                        type_info = node_text(t, source);
                    }
                }
                items.push(HdlPackageItem {
                    name: item_name,
                    kind: "subtype".to_string(),
                    type_info,
                    value: String::new(),
                    parameters: Vec::new(),
                    return_type: String::new(),
                    doc: item_doc(c, source),
                });
            }
            "constant_declaration" => {
                let mut names = Vec::new();
                let mut type_info = String::new();
                let mut value = String::new();
                for j in 0..c.child_count() {
                    let Some(gc) = c.child(j) else { continue };
                    match gc.kind() {
                        "identifier_list" => {
                            for k in 0..gc.child_count() {
                                if let Some(id) = gc.child(k) {
                                    // See decode_vhdl_interface_decl: match any
                                    // named child, not a fixed identifier kind.
                                    if id.is_named() {
                                        names.push(node_text(id, source));
                                    }
                                }
                            }
                        }
                        "subtype_indication" => {
                            if let Some(t) = gc.child_by_field_name("type") {
                                type_info = node_text(t, source);
                            }
                        }
                        "initialiser" => {
                            for k in 0..gc.child_count() {
                                if let Some(v) = gc.child(k) {
                                    if v.is_named() {
                                        value = node_text(v, source);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                let doc = item_doc(c, source);
                for n in names {
                    if items.len() >= MAX_PACKAGE_ITEMS {
                        break;
                    }
                    items.push(HdlPackageItem {
                        name: n,
                        kind: "constant".to_string(),
                        type_info: type_info.clone(),
                        value: value.clone(),
                        parameters: Vec::new(),
                        return_type: String::new(),
                        doc: doc.clone(),
                    });
                }
            }
            "subprogram_declaration" => {
                for j in 0..c.child_count() {
                    if let Some(gc) = c.child(j) {
                        if gc.kind() == "function_specification" || gc.kind() == "procedure_specification"
                        {
                            items.push(decode_vhdl_subprogram(gc, source));
                        }
                    }
                }
            }
            _ => {}
        }
    }
    HdlPackage { name, items }
}

/// Find every package defined anywhere in a parsed VHDL tree.
pub fn find_all_vhdl_packages(tree: &Tree, source: &str) -> Vec<HdlPackage> {
    find_all_of_kind(tree.root_node(), "package_declaration", MAX_DEFINITIONS)
        .into_iter()
        .map(|n| extract_vhdl_package(n, source))
        .collect()
}

// ─── Cross-dialect helpers shared by the "named query" nodes ──────────────

/// Resolve `source`'s language (hint-or-detect) and parse it into every
/// module/entity definition (Verilog modules or VHDL entities — the shared
/// HdlModule shape). Returns (language used, definitions found).
pub fn find_definitions(text: &str, hint: &str) -> Result<(String, Vec<HdlModule>), String> {
    let language = resolve_language(text, hint)?;
    let modules = match language.as_str() {
        "verilog" => {
            let tree = parse_verilog(text)?;
            find_all_verilog_modules(&tree, text)
        }
        "vhdl" => {
            let tree = parse_vhdl(text)?;
            find_all_vhdl_entities(&tree, text)
        }
        _ => Vec::new(),
    };
    Ok((language, modules))
}

/// Pick one definition by name from a list; empty name means "the first
/// definition found in source order".
pub fn pick_definition(defs: Vec<HdlModule>, name: &str) -> Option<HdlModule> {
    if name.is_empty() {
        defs.into_iter().next()
    } else {
        defs.into_iter().find(|m| m.name == name)
    }
}
