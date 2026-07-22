use crate::axiom_context::AxiomContext;
use crate::gen::messages::{NamedHdlQuery, PortList};

#[path = "hdlutil.rs"]
mod hdlutil;

/// Extract only the output-direction ports of a named module/entity (or the
/// first one found if name is empty) — the signals a caller should observe.
/// A filtered convenience view over ExtractPorts for testbench/monitor
/// generation.
pub fn extract_output_ports(
    ax: &dyn AxiomContext,
    input: NamedHdlQuery,
) -> Result<PortList, Box<dyn std::error::Error>> {
    let _ = ax;
    let source = input.source.unwrap_or_default();
    let (language, defs) = match hdlutil::find_definitions(&source.text, &source.language) {
        Ok(v) => v,
        Err(e) => return Ok(PortList { error: e, ..Default::default() }),
    };
    match hdlutil::pick_definition(defs, &input.name) {
        Some(module) => {
            let has_error = module.has_error;
            let ports: Vec<_> = module.ports.into_iter().filter(|p| p.direction == "output").collect();
            Ok(PortList {
                language,
                module_name: module.name,
                found: true,
                count: ports.len() as u32,
                ports,
                error: String::new(),
                has_error,
            })
        }
        None => Ok(PortList { language, found: false, ..Default::default() }),
    }
}
