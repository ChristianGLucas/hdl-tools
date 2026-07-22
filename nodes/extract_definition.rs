use crate::axiom_context::AxiomContext;
use crate::gen::messages::{HdlInterface, NamedHdlQuery};

#[path = "hdlutil.rs"]
mod hdlutil;

/// Extract one named module (Verilog/SystemVerilog) or entity (VHDL) by name
/// from a source and return its full interface — every port and every
/// parameter/generic. found is false if the name is not defined in the
/// source.
pub fn extract_definition(
    ax: &dyn AxiomContext,
    input: NamedHdlQuery,
) -> Result<HdlInterface, Box<dyn std::error::Error>> {
    let _ = ax;
    let source = input.source.unwrap_or_default();
    let (language, defs) = match hdlutil::find_definitions(&source.text, &source.language) {
        Ok(v) => v,
        Err(e) => return Ok(HdlInterface { error: e, ..Default::default() }),
    };
    match hdlutil::pick_definition(defs, &input.name) {
        Some(module) => Ok(HdlInterface { language, found: true, module: Some(module), error: String::new() }),
        None => Ok(HdlInterface { language, found: false, module: None, error: String::new() }),
    }
}
