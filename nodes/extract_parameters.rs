use crate::axiom_context::AxiomContext;
use crate::gen::messages::{NamedHdlQuery, ParamList};

#[path = "hdlutil.rs"]
mod hdlutil;

/// Extract the parameters (Verilog/SystemVerilog "parameter") or generics
/// (VHDL "generic") of a named module/entity — name, declared type, and
/// default value — for a named module/entity (or the first one found if name
/// is empty).
pub fn extract_parameters(
    ax: &dyn AxiomContext,
    input: NamedHdlQuery,
) -> Result<ParamList, Box<dyn std::error::Error>> {
    let _ = ax;
    let source = input.source.unwrap_or_default();
    let (language, defs) = match hdlutil::find_definitions(&source.text, &source.language) {
        Ok(v) => v,
        Err(e) => return Ok(ParamList { error: e, ..Default::default() }),
    };
    match hdlutil::pick_definition(defs, &input.name) {
        Some(module) => Ok(ParamList {
            language,
            module_name: module.name,
            found: true,
            count: module.parameters.len() as u32,
            parameters: module.parameters,
            error: String::new(),
        }),
        None => Ok(ParamList { language, found: false, ..Default::default() }),
    }
}
