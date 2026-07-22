// Tests for the GenerateInstantiationTemplate node.
#[cfg(test)]
#[path = "testctx.rs"]
mod testctx;
#[cfg(test)]
#[path = "fixtures.rs"]
mod fixtures;

#[cfg(test)]
mod tests {
    use crate::gen::messages::{HdlSource, NamedHdlQuery};
    use crate::generate_instantiation_template::generate_instantiation_template;
    use super::fixtures::*;
    use super::testctx::test_context;

    fn query(text: &str, name: &str) -> NamedHdlQuery {
        NamedHdlQuery {
            source: Some(HdlSource { text: text.to_string(), language: String::new() }),
            name: name.to_string(),
        }
    }

    // The STRUCTURED connection lists are hand-derived facts (real oracle);
    // the snippet text is our own convenience rendering with no external
    // format spec to check byte-for-byte, so it is checked for containing
    // every required piece rather than an exact string match.
    #[test]
    fn test_verilog_instantiation_template() {
        let ax = test_context();
        let out = generate_instantiation_template(&ax, query(VERILOG_COUNTER, "counter")).unwrap();
        assert_eq!(out.error, "");
        assert!(out.found);
        assert_eq!(out.language, "verilog");
        assert_eq!(out.module_name, "counter");

        assert_eq!(out.port_connections.len(), 3);
        assert_eq!(out.port_connections[0].port_name, "clk");
        assert_eq!(out.port_connections[0].direction, "input");
        assert_eq!(out.port_connections[0].suggested_signal, "clk");
        assert_eq!(out.port_connections[2].port_name, "count");
        assert_eq!(out.port_connections[2].direction, "output");

        assert_eq!(out.param_connections.len(), 2);
        assert_eq!(out.param_connections[0].name, "WIDTH");
        assert_eq!(out.param_connections[0].default_value, "8");
        assert_eq!(out.param_connections[1].name, "STEP");
        assert_eq!(out.param_connections[1].default_value, "1");

        assert!(out.snippet.contains("counter"));
        assert!(out.snippet.contains(".clk(clk)"));
        assert!(out.snippet.contains(".rst_n(rst_n)"));
        assert!(out.snippet.contains(".count(count)"));
        assert!(out.snippet.contains(".WIDTH(8)"));
        assert!(out.snippet.contains(".STEP(1)"));
    }

    #[test]
    fn test_vhdl_instantiation_template() {
        let ax = test_context();
        let out = generate_instantiation_template(&ax, query(VHDL_COUNTER, "counter")).unwrap();
        assert_eq!(out.language, "vhdl");
        assert_eq!(out.port_connections.len(), 3);
        assert_eq!(out.param_connections.len(), 2);
        assert!(out.snippet.contains("entity work.counter"));
        assert!(out.snippet.contains("clk => clk"));
        assert!(out.snippet.contains("WIDTH => 8"));
    }

    // A module with no parameters omits the override block entirely rather
    // than emitting an empty one.
    #[test]
    fn test_no_parameters_omits_override_block() {
        let ax = test_context();
        let out = generate_instantiation_template(&ax, query(VERILOG_NONANSI, "adder")).unwrap();
        assert!(out.found);
        assert!(out.param_connections.is_empty());
        assert!(!out.snippet.contains("#("));
    }

    #[test]
    fn test_not_found() {
        let ax = test_context();
        let out = generate_instantiation_template(&ax, query(VERILOG_COUNTER, "nope")).unwrap();
        assert_eq!(out.error, "");
        assert!(!out.found);
        assert_eq!(out.snippet, "");
    }

    // Regression (found by adversarial review): a malformed port list inside
    // the picked entity must surface has_error so a caller does not blindly
    // trust a snippet built from a wrong direction / dropped port.
    #[test]
    fn test_malformed_port_list_surfaces_has_error() {
        let ax = test_context();
        let out =
            generate_instantiation_template(&ax, query(VHDL_PORT_LIST_MISSING_SEMICOLON, "top")).unwrap();
        assert!(out.found);
        assert!(out.has_error);
    }
}
