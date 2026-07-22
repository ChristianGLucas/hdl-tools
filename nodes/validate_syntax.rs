use crate::axiom_context::AxiomContext;
use crate::gen::messages::{HdlSource, SyntaxValidation};

#[path = "hdlutil.rs"]
mod hdlutil;

/// Validate that HDL source parses cleanly and report every syntax issue
/// found (ERROR or MISSING regions with byte/row/column spans). ok is true
/// only when there are no issues; this never fails on malformed input — it
/// reports the problem structurally. Uses the language hint if given,
/// otherwise auto-detects.
pub fn validate_syntax(
    ax: &dyn AxiomContext,
    input: HdlSource,
) -> Result<SyntaxValidation, Box<dyn std::error::Error>> {
    let _ = ax;
    let language = match hdlutil::resolve_language(&input.text, &input.language) {
        Ok(l) => l,
        Err(e) => return Ok(SyntaxValidation { error: e, ..Default::default() }),
    };
    let tree = match language.as_str() {
        "vhdl" => hdlutil::parse_vhdl(&input.text),
        _ => hdlutil::parse_verilog(&input.text),
    };
    let tree = match tree {
        Ok(t) => t,
        Err(e) => return Ok(SyntaxValidation { error: e, ..Default::default() }),
    };
    let issues = hdlutil::collect_issues(&tree);
    Ok(SyntaxValidation {
        language,
        ok: issues.is_empty(),
        issue_count: issues.len() as u32,
        issues,
        error: String::new(),
    })
}
