// Tests for the ExtractDefinition node.
#[cfg(test)]
#[path = "testctx.rs"]
mod testctx;
#[cfg(test)]
#[path = "fixtures.rs"]
mod fixtures;

#[cfg(test)]
mod tests {
    use crate::extract_definition::extract_definition;
    use crate::gen::messages::{HdlSource, NamedHdlQuery};
    use super::fixtures::*;
    use super::testctx::test_context;

    fn query(text: &str, name: &str) -> NamedHdlQuery {
        NamedHdlQuery {
            source: Some(HdlSource { text: text.to_string(), language: String::new() }),
            name: name.to_string(),
        }
    }

    #[test]
    fn test_extract_by_name() {
        let ax = test_context();
        let out = extract_definition(&ax, query(VERILOG_TWO_MODULES, "second_mod")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.language, "verilog");
        assert!(out.found);
        let m = out.module.unwrap();
        assert_eq!(m.name, "second_mod");
        assert_eq!(m.ports.len(), 2);
    }

    // Empty name means "the first definition found in source order".
    #[test]
    fn test_empty_name_returns_first() {
        let ax = test_context();
        let out = extract_definition(&ax, query(VERILOG_TWO_MODULES, "")).unwrap();
        assert!(out.found);
        assert_eq!(out.module.unwrap().name, "first_mod");
    }

    #[test]
    fn test_unknown_name_not_found_not_error() {
        let ax = test_context();
        let out = extract_definition(&ax, query(VERILOG_COUNTER, "does_not_exist")).unwrap();
        assert_eq!(out.error, "");
        assert!(!out.found);
        assert!(out.module.is_none());
    }

    #[test]
    fn test_vhdl_entity_by_name() {
        let ax = test_context();
        let out = extract_definition(&ax, query(VHDL_COUNTER, "counter")).unwrap();
        assert_eq!(out.language, "vhdl");
        assert!(out.found);
        let m = out.module.unwrap();
        assert_eq!(m.kind, "entity");
        assert_eq!(m.parameters.len(), 2);
    }
}
