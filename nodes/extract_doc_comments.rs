use crate::axiom_context::AxiomContext;
use crate::gen::messages::{DocCommentSummary, NamedHdlQuery, PortDoc};

#[path = "hdlutil.rs"]
mod hdlutil;

/// Extract the documentation comments associated with a named module/entity
/// and its ports — the comment immediately above the declaration, and any
/// comment directly attached to each port (leading or same-line trailing).
/// Ports with no comment are omitted from port_docs.
pub fn extract_doc_comments(
    ax: &dyn AxiomContext,
    input: NamedHdlQuery,
) -> Result<DocCommentSummary, Box<dyn std::error::Error>> {
    let _ = ax;
    let source = input.source.unwrap_or_default();
    let (language, defs) = match hdlutil::find_definitions(&source.text, &source.language) {
        Ok(v) => v,
        Err(e) => return Ok(DocCommentSummary { error: e, ..Default::default() }),
    };
    match hdlutil::pick_definition(defs, &input.name) {
        Some(module) => {
            let port_docs: Vec<PortDoc> = module
                .ports
                .into_iter()
                .filter(|p| !p.doc.is_empty())
                .map(|p| PortDoc { port_name: p.name, doc: p.doc })
                .collect();
            Ok(DocCommentSummary {
                language,
                module_name: module.name,
                found: true,
                module_doc: module.doc,
                port_docs,
                error: String::new(),
            })
        }
        None => Ok(DocCommentSummary { language, found: false, ..Default::default() }),
    }
}
