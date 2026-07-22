use crate::axiom_context::AxiomContext;
use crate::gen::messages::{HdlSource, VhdlEntityList};

#[path = "hdlutil.rs"]
mod hdlutil;

/// Parse VHDL source into every entity it declares, each with its full port
/// list (name, mode/direction, type, vector range) and generic list (name,
/// type, default). Handles standalone entity declarations directly (not only
/// the legacy component-mirror idiom). Malformed source still returns
/// whatever entities could be recovered, with has_error set.
pub fn parse_vhdl_entities(
    ax: &dyn AxiomContext,
    input: HdlSource,
) -> Result<VhdlEntityList, Box<dyn std::error::Error>> {
    let _ = ax;
    let tree = match hdlutil::parse_vhdl(&input.text) {
        Ok(t) => t,
        Err(e) => return Ok(VhdlEntityList { error: e, ..Default::default() }),
    };
    let has_error = tree.root_node().has_error();
    let entities = hdlutil::find_all_vhdl_entities(&tree, &input.text);
    Ok(VhdlEntityList {
        entity_count: entities.len() as u32,
        entities,
        has_error,
        error: String::new(),
    })
}
