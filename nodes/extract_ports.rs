use crate::axiom_context::AxiomContext;
use crate::gen::messages::{NamedHdlQuery, PortList};

#[path = "hdlutil.rs"]
mod hdlutil;

/// Extract the full port list — name, direction, data type, and bit
/// width/vector range — for a named module/entity (or the first one found if
/// name is empty).
pub fn extract_ports(
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
        Some(module) => Ok(PortList {
            language,
            module_name: module.name,
            found: true,
            count: module.ports.len() as u32,
            ports: module.ports,
            error: String::new(),
        }),
        None => Ok(PortList { language, found: false, ..Default::default() }),
    }
}
