use crate::axiom_context::AxiomContext;
use crate::gen::messages::{HdlSource, LanguageDetection};

#[path = "hdlutil.rs"]
mod hdlutil;

/// Detect whether HDL source text is Verilog/SystemVerilog or VHDL by parsing
/// it with both grammars and comparing syntax-error counts (the grammar that
/// parses cleaner wins) — an objective, parser-verified signal rather than a
/// keyword guess. Returns a confidence score and both error counts; empty
/// language means neither grammar made sense of the input (or both parsed
/// equally cleanly, e.g. trivial/near-empty input).
pub fn detect_language(
    ax: &dyn AxiomContext,
    input: HdlSource,
) -> Result<LanguageDetection, Box<dyn std::error::Error>> {
    let _ = ax;
    let d = match hdlutil::detect_language(&input.text) {
        Ok(d) => d,
        Err(e) => return Ok(LanguageDetection { error: e, ..Default::default() }),
    };
    Ok(LanguageDetection {
        language: d.language,
        confidence: d.confidence,
        verilog_error_nodes: d.verilog_errors,
        vhdl_error_nodes: d.vhdl_errors,
        error: String::new(),
    })
}
