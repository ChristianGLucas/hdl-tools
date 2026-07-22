use crate::axiom_context::AxiomContext;
use crate::gen::messages::{HdlModule, InstantiationTemplate, NamedHdlQuery, ParamConnection, PortConnection};

#[path = "hdlutil.rs"]
mod hdlutil;

fn verilog_snippet(m: &HdlModule) -> String {
    let inst_name = format!("{}_inst", m.name);
    let mut s = String::new();
    s.push_str(&m.name);
    if !m.parameters.is_empty() {
        s.push_str(" #(\n");
        let lines: Vec<String> =
            m.parameters.iter().map(|p| format!("    .{}({})", p.name, p.default_value)).collect();
        s.push_str(&lines.join(",\n"));
        s.push_str("\n  )");
    }
    s.push(' ');
    s.push_str(&inst_name);
    s.push_str(" (\n");
    let lines: Vec<String> = m.ports.iter().map(|p| format!("    .{}({})", p.name, p.name)).collect();
    s.push_str(&lines.join(",\n"));
    s.push_str("\n  );\n");
    s
}

fn vhdl_snippet(m: &HdlModule) -> String {
    let inst_name = format!("{}_inst", m.name);
    let mut s = String::new();
    s.push_str(&format!("{}_inst : entity work.{}\n", m.name, m.name));
    if !m.parameters.is_empty() {
        s.push_str("  generic map (\n");
        let lines: Vec<String> =
            m.parameters.iter().map(|p| format!("    {} => {}", p.name, p.default_value)).collect();
        s.push_str(&lines.join(",\n"));
        s.push_str("\n  )\n");
    }
    s.push_str("  port map (\n");
    let lines: Vec<String> = m.ports.iter().map(|p| format!("    {} => {}", p.name, p.name)).collect();
    s.push_str(&lines.join(",\n"));
    s.push_str("\n  );\n");
    let _ = inst_name;
    s
}

/// Generate a ready-to-copy instantiation template for a named module/entity:
/// a named-port Verilog/SystemVerilog instantiation (with a parameter
/// override block if it has parameters) or a VHDL component declaration plus
/// port-map/generic-map instantiation, along with the structured port and
/// parameter connection lists the snippet is built from — the "how do I wire
/// this up" node. Suggested signal names follow the common same-name-as-port
/// convention; edit them to fit your own signal naming.
pub fn generate_instantiation_template(
    ax: &dyn AxiomContext,
    input: NamedHdlQuery,
) -> Result<InstantiationTemplate, Box<dyn std::error::Error>> {
    let _ = ax;
    let source = input.source.unwrap_or_default();
    let (language, defs) = match hdlutil::find_definitions(&source.text, &source.language) {
        Ok(v) => v,
        Err(e) => return Ok(InstantiationTemplate { error: e, ..Default::default() }),
    };
    let module = match hdlutil::pick_definition(defs, &input.name) {
        Some(m) => m,
        None => return Ok(InstantiationTemplate { language, found: false, ..Default::default() }),
    };
    let snippet = match language.as_str() {
        "vhdl" => vhdl_snippet(&module),
        _ => verilog_snippet(&module),
    };
    let port_connections: Vec<PortConnection> = module
        .ports
        .iter()
        .map(|p| PortConnection {
            port_name: p.name.clone(),
            direction: p.direction.clone(),
            suggested_signal: p.name.clone(),
        })
        .collect();
    let param_connections: Vec<ParamConnection> = module
        .parameters
        .iter()
        .map(|p| ParamConnection { name: p.name.clone(), default_value: p.default_value.clone() })
        .collect();
    Ok(InstantiationTemplate {
        language,
        module_name: module.name,
        found: true,
        snippet,
        port_connections,
        param_connections,
        error: String::new(),
    })
}
