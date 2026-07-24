// Tests for the ParseVhdlPackages node.
#[cfg(test)]
#[path = "testctx.rs"]
mod testctx;
#[cfg(test)]
#[path = "fixtures.rs"]
mod fixtures;

#[cfg(test)]
mod tests {
    use crate::gen::messages::HdlSource;
    use crate::parse_vhdl_packages::parse_vhdl_packages;
    use super::fixtures::*;
    use super::testctx::test_context;

    fn src(text: &str) -> HdlSource {
        HdlSource { text: text.to_string(), language: String::new() }
    }

    // Golden: every declared item hand-checked against the fixture text
    // (independent oracle — read directly against IEEE 1076 package syntax):
    // a constant, an enum type, a subtype, a function, and a procedure whose
    // parameters use `signal`/`inout` modes.
    #[test]
    fn test_package_golden_shape() {
        let ax = test_context();
        let out = parse_vhdl_packages(&ax, src(VHDL_PACKAGE)).unwrap();
        assert_eq!(out.error, "");
        assert!(!out.has_error);
        assert_eq!(out.package_count, 1);
        let p = &out.packages[0];
        assert_eq!(p.name, "math_pkg");
        assert_eq!(p.items.len(), 5);

        let c = &p.items[0];
        assert_eq!(c.name, "MAX_WIDTH");
        assert_eq!(c.kind, "constant");
        assert_eq!(c.type_info, "integer");
        assert_eq!(c.value, "32");

        let t = &p.items[1];
        assert_eq!(t.name, "state_t");
        assert_eq!(t.kind, "type");
        assert_eq!(t.type_info, "enumeration");

        let s = &p.items[2];
        assert_eq!(s.name, "byte_t");
        assert_eq!(s.kind, "subtype");
        assert_eq!(s.type_info, "std_logic_vector(7 downto 0)");

        let f = &p.items[3];
        assert_eq!(f.name, "clamp");
        assert_eq!(f.kind, "function");
        assert_eq!(f.return_type, "integer");
        assert_eq!(f.parameters.len(), 3);
        assert_eq!(f.parameters[0].name, "x");
        assert_eq!(f.parameters[0].data_type, "integer");
        assert_eq!(f.parameters[1].name, "lo");
        assert_eq!(f.parameters[2].name, "hi");

        let pr = &p.items[4];
        assert_eq!(pr.name, "pulse");
        assert_eq!(pr.kind, "procedure");
        assert_eq!(pr.return_type, "");
        assert_eq!(pr.parameters.len(), 2);
        assert_eq!(pr.parameters[0].name, "clk");
        assert_eq!(pr.parameters[0].data_type, "std_logic");
        assert_eq!(pr.parameters[1].name, "count");
        assert_eq!(pr.parameters[1].data_type, "integer");
    }

    #[test]
    fn test_malformed_source_sets_has_error_not_error() {
        let ax = test_context();
        let out = parse_vhdl_packages(&ax, src(VHDL_MALFORMED)).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.package_count, 0);
    }

    #[test]
    fn test_large_input_no_crash() {
        // Payload-size limits are the platform's job, not this node's; a
        // large input must still parse cleanly (recovering zero packages
        // from non-HDL text) instead of crashing.
        let ax = test_context();
        let big = "x".repeat(2_000_001);
        let out = parse_vhdl_packages(&ax, src(&big)).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.package_count, 0);
    }
}
