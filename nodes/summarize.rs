use crate::axiom_context::AxiomContext;
use crate::gen::messages::{HdlSource, HdlSummary};

#[path = "hdlutil.rs"]
mod hdlutil;

/// Compute summary counts for an HDL source — number of modules/entities/
/// packages found and total ports/parameters across them — plus the detected
/// language. A compact structural fingerprint without extracting every full
/// interface. package_count only ever counts VHDL packages (this package does
/// not extract SystemVerilog `package` blocks); port_count/param_count sum
/// only module/entity ports and parameters/generics, not package-internal
/// subprogram parameters.
pub fn summarize(
    ax: &dyn AxiomContext,
    input: HdlSource,
) -> Result<HdlSummary, Box<dyn std::error::Error>> {
    let _ = ax;
    let language = match hdlutil::resolve_language(&input.text, &input.language) {
        Ok(l) => l,
        Err(e) => return Ok(HdlSummary { error: e, ..Default::default() }),
    };
    match language.as_str() {
        "verilog" => {
            let tree = match hdlutil::parse_verilog(&input.text) {
                Ok(t) => t,
                Err(e) => return Ok(HdlSummary { error: e, ..Default::default() }),
            };
            let has_error = tree.root_node().has_error();
            let modules = hdlutil::find_all_verilog_modules(&tree, &input.text);
            let port_count: u32 = modules.iter().map(|m| m.ports.len() as u32).sum();
            let param_count: u32 = modules.iter().map(|m| m.parameters.len() as u32).sum();
            Ok(HdlSummary {
                language,
                module_count: modules.len() as u32,
                entity_count: 0,
                package_count: 0,
                port_count,
                param_count,
                has_error,
                error: String::new(),
            })
        }
        "vhdl" => {
            let tree = match hdlutil::parse_vhdl(&input.text) {
                Ok(t) => t,
                Err(e) => return Ok(HdlSummary { error: e, ..Default::default() }),
            };
            let has_error = tree.root_node().has_error();
            let entities = hdlutil::find_all_vhdl_entities(&tree, &input.text);
            let packages = hdlutil::find_all_vhdl_packages(&tree, &input.text);
            let port_count: u32 = entities.iter().map(|m| m.ports.len() as u32).sum();
            let param_count: u32 = entities.iter().map(|m| m.parameters.len() as u32).sum();
            Ok(HdlSummary {
                language,
                module_count: 0,
                entity_count: entities.len() as u32,
                package_count: packages.len() as u32,
                port_count,
                param_count,
                has_error,
                error: String::new(),
            })
        }
        _ => Ok(HdlSummary { language, ..Default::default() }),
    }
}
