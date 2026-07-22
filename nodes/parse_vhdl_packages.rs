use crate::axiom_context::AxiomContext;
use crate::gen::messages::{HdlSource, VhdlPackageList};

#[path = "hdlutil.rs"]
mod hdlutil;

/// Parse VHDL source into every package it declares, each with its
/// directly-declared types, subtypes, constants, and function/procedure
/// signatures. Malformed source still returns whatever packages could be
/// recovered, with has_error set.
pub fn parse_vhdl_packages(
    ax: &dyn AxiomContext,
    input: HdlSource,
) -> Result<VhdlPackageList, Box<dyn std::error::Error>> {
    let _ = ax;
    let tree = match hdlutil::parse_vhdl(&input.text) {
        Ok(t) => t,
        Err(e) => return Ok(VhdlPackageList { error: e, ..Default::default() }),
    };
    let has_error = tree.root_node().has_error();
    let packages = hdlutil::find_all_vhdl_packages(&tree, &input.text);
    Ok(VhdlPackageList {
        package_count: packages.len() as u32,
        packages,
        has_error,
        error: String::new(),
    })
}
