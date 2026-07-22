// Tests for the ExtractPorts node.
#[cfg(test)]
#[path = "testctx.rs"]
mod testctx;
#[cfg(test)]
#[path = "fixtures.rs"]
mod fixtures;

#[cfg(test)]
mod tests {
    use crate::extract_ports::extract_ports;
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
    fn test_verilog_ports() {
        let ax = test_context();
        let out = extract_ports(&ax, query(VERILOG_COUNTER, "counter")).unwrap();
        assert_eq!(out.error, "");
        assert!(out.found);
        assert!(!out.has_error);
        assert_eq!(out.module_name, "counter");
        assert_eq!(out.count, 3);
        assert_eq!(out.ports[2].name, "count");
        assert_eq!(out.ports[2].width, "[WIDTH-1:0]");
    }

    #[test]
    fn test_vhdl_ports() {
        let ax = test_context();
        let out = extract_ports(&ax, query(VHDL_COUNTER, "")).unwrap();
        assert!(out.found);
        assert!(!out.has_error);
        assert_eq!(out.count, 3);
        assert_eq!(out.ports[2].width, "(WIDTH-1 downto 0)");
    }

    // Regression (found by adversarial review): a missing semicolon between
    // two port declarations makes the grammar's error recovery absorb the
    // second port into an ERROR node, which can silently flip the first
    // port's direction and drop the second. The entity is still `found`
    // (its name parses fine) but has_error MUST be true so a caller knows
    // not to trust the extracted ports without independent verification.
    #[test]
    fn test_malformed_port_list_surfaces_has_error() {
        let ax = test_context();
        let out = extract_ports(&ax, query(VHDL_PORT_LIST_MISSING_SEMICOLON, "top")).unwrap();
        assert_eq!(out.error, "");
        assert!(out.found);
        assert!(out.has_error, "a syntax error inside the picked entity's own port list must be surfaced");
    }

    #[test]
    fn test_not_found() {
        let ax = test_context();
        let out = extract_ports(&ax, query(VERILOG_COUNTER, "nope")).unwrap();
        assert_eq!(out.error, "");
        assert!(!out.found);
        assert_eq!(out.count, 0);
        assert!(out.ports.is_empty());
    }
}
