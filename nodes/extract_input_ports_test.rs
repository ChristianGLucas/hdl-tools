// Tests for the ExtractInputPorts node.
#[cfg(test)]
#[path = "testctx.rs"]
mod testctx;
#[cfg(test)]
#[path = "fixtures.rs"]
mod fixtures;

#[cfg(test)]
mod tests {
    use crate::extract_input_ports::extract_input_ports;
    use crate::gen::messages::{HdlSource, NamedHdlQuery};
    use super::fixtures::*;
    use super::testctx::test_context;

    fn query(text: &str, name: &str) -> NamedHdlQuery {
        NamedHdlQuery {
            source: Some(HdlSource { text: text.to_string(), language: String::new() }),
            name: name.to_string(),
        }
    }

    // counter has 2 inputs (clk, rst_n) and 1 output (count) — only the
    // inputs must come back, in source order.
    #[test]
    fn test_filters_to_inputs_only() {
        let ax = test_context();
        let out = extract_input_ports(&ax, query(VERILOG_COUNTER, "counter")).unwrap();
        assert_eq!(out.error, "");
        assert!(out.found);
        assert_eq!(out.count, 2);
        assert_eq!(out.ports[0].name, "clk");
        assert_eq!(out.ports[1].name, "rst_n");
        assert!(out.ports.iter().all(|p| p.direction == "input"));
    }

    #[test]
    fn test_vhdl_inputs() {
        let ax = test_context();
        let out = extract_input_ports(&ax, query(VHDL_COUNTER, "counter")).unwrap();
        assert_eq!(out.count, 2);
        assert!(out.ports.iter().all(|p| p.direction == "input"));
    }

    #[test]
    fn test_not_found() {
        let ax = test_context();
        let out = extract_input_ports(&ax, query(VERILOG_COUNTER, "nope")).unwrap();
        assert!(!out.found);
        assert_eq!(out.count, 0);
    }
}
