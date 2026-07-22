use crate::axiom_context::AxiomContext;
use crate::gen::messages::{DefinitionList, HdlSource};

#[path = "hdlutil.rs"]
mod hdlutil;

/// List the names of every module/entity defined in an HDL source,
/// auto-detecting Verilog/SystemVerilog vs VHDL (or honoring an explicit
/// language hint). The cheapest way to answer "what does this file define"
/// before extracting a specific interface.
pub fn list_definitions(
    ax: &dyn AxiomContext,
    input: HdlSource,
) -> Result<DefinitionList, Box<dyn std::error::Error>> {
    let _ = ax;
    let language = match hdlutil::resolve_language(&input.text, &input.language) {
        Ok(l) => l,
        Err(e) => return Ok(DefinitionList { error: e, ..Default::default() }),
    };
    let names: Vec<String> = match language.as_str() {
        "verilog" => {
            let tree = match hdlutil::parse_verilog(&input.text) {
                Ok(t) => t,
                Err(e) => return Ok(DefinitionList { error: e, ..Default::default() }),
            };
            hdlutil::find_all_verilog_modules(&tree, &input.text)
                .into_iter()
                .map(|m| m.name)
                .collect()
        }
        "vhdl" => {
            let tree = match hdlutil::parse_vhdl(&input.text) {
                Ok(t) => t,
                Err(e) => return Ok(DefinitionList { error: e, ..Default::default() }),
            };
            hdlutil::find_all_vhdl_entities(&tree, &input.text)
                .into_iter()
                .map(|m| m.name)
                .collect()
        }
        _ => Vec::new(),
    };
    Ok(DefinitionList {
        language,
        count: names.len() as u32,
        names,
        error: String::new(),
    })
}
