// Tests for the ExtractOutputPorts node.
#[cfg(test)]
#[path = "testctx.rs"]
mod testctx;
#[cfg(test)]
#[path = "fixtures.rs"]
mod fixtures;

#[cfg(test)]
mod tests {
    use crate::extract_output_ports::extract_output_ports;
    use crate::gen::messages::{HdlSource, NamedHdlQuery};
    use super::fixtures::*;
    use super::testctx::test_context;

    fn query(text: &str, name: &str) -> NamedHdlQuery {
        NamedHdlQuery {
            source: Some(HdlSource { text: text.to_string(), language: String::new() }),
            name: name.to_string(),
        }
    }

    // counter has exactly 1 output (count) among its 3 ports.
    #[test]
    fn test_filters_to_outputs_only() {
        let ax = test_context();
        let out = extract_output_ports(&ax, query(VERILOG_COUNTER, "counter")).unwrap();
        assert_eq!(out.error, "");
        assert!(out.found);
        assert_eq!(out.count, 1);
        assert_eq!(out.ports[0].name, "count");
        assert_eq!(out.ports[0].direction, "output");
    }

    #[test]
    fn test_vhdl_outputs() {
        let ax = test_context();
        let out = extract_output_ports(&ax, query(VHDL_COUNTER, "counter")).unwrap();
        assert_eq!(out.count, 1);
        assert_eq!(out.ports[0].name, "count");
    }

    #[test]
    fn test_not_found() {
        let ax = test_context();
        let out = extract_output_ports(&ax, query(VERILOG_COUNTER, "nope")).unwrap();
        assert!(!out.found);
        assert_eq!(out.count, 0);
    }
}
