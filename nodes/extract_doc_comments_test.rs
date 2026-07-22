// Tests for the ExtractDocComments node.
#[cfg(test)]
#[path = "testctx.rs"]
mod testctx;
#[cfg(test)]
#[path = "fixtures.rs"]
mod fixtures;

#[cfg(test)]
mod tests {
    use crate::extract_doc_comments::extract_doc_comments;
    use crate::gen::messages::{HdlSource, NamedHdlQuery};
    use super::fixtures::*;
    use super::testctx::test_context;

    fn query(text: &str, name: &str) -> NamedHdlQuery {
        NamedHdlQuery {
            source: Some(HdlSource { text: text.to_string(), language: String::new() }),
            name: name.to_string(),
        }
    }

    // counter has a module-level doc and two ports with trailing doc
    // comments; the third port (count) has none and must be OMITTED, not
    // included with an empty string.
    #[test]
    fn test_verilog_doc_comments() {
        let ax = test_context();
        let out = extract_doc_comments(&ax, query(VERILOG_COUNTER, "counter")).unwrap();
        assert_eq!(out.error, "");
        assert!(out.found);
        assert_eq!(out.module_doc, "A simple up counter with synchronous reset.");
        assert_eq!(out.port_docs.len(), 2);
        assert_eq!(out.port_docs[0].port_name, "clk");
        assert_eq!(out.port_docs[0].doc, "system clock");
        assert_eq!(out.port_docs[1].port_name, "rst_n");
        assert_eq!(out.port_docs[1].doc, "active-low reset");
    }

    #[test]
    fn test_vhdl_doc_comments() {
        let ax = test_context();
        let out = extract_doc_comments(&ax, query(VHDL_COUNTER, "counter")).unwrap();
        assert_eq!(out.module_doc, "A simple up counter with synchronous reset.");
        assert_eq!(out.port_docs.len(), 2);
        assert_eq!(out.port_docs[0].doc, "system clock");
    }

    // No doc comments anywhere: module_doc empty, port_docs empty (not an
    // error).
    #[test]
    fn test_no_comments_is_empty_not_error() {
        let ax = test_context();
        let out = extract_doc_comments(&ax, query(VERILOG_NONANSI, "adder")).unwrap();
        assert_eq!(out.error, "");
        assert!(out.found);
        assert_eq!(out.module_doc, "");
        assert!(out.port_docs.is_empty());
    }
}
