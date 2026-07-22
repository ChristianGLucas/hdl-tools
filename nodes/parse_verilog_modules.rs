use crate::axiom_context::AxiomContext;
use crate::gen::messages::{HdlSource, VerilogModuleList};

#[path = "hdlutil.rs"]
mod hdlutil;

/// Parse Verilog or SystemVerilog source (IEEE 1800-2017 grammar) into every
/// module it defines, each with its full port list (name, direction, type,
/// bit width) and parameter list (name, type, default). Malformed source
/// still returns whatever modules could be recovered, with has_error set.
pub fn parse_verilog_modules(
    ax: &dyn AxiomContext,
    input: HdlSource,
) -> Result<VerilogModuleList, Box<dyn std::error::Error>> {
    let _ = ax;
    let tree = match hdlutil::parse_verilog(&input.text) {
        Ok(t) => t,
        Err(e) => return Ok(VerilogModuleList { error: e, ..Default::default() }),
    };
    let has_error = tree.root_node().has_error();
    let modules = hdlutil::find_all_verilog_modules(&tree, &input.text);
    Ok(VerilogModuleList {
        module_count: modules.len() as u32,
        modules,
        has_error,
        error: String::new(),
    })
}
