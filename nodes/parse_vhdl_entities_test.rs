// Tests for the ParseVhdlEntities node.
#[cfg(test)]
#[path = "testctx.rs"]
mod testctx;
#[cfg(test)]
#[path = "fixtures.rs"]
mod fixtures;

#[cfg(test)]
mod tests {
    use crate::gen::messages::HdlSource;
    use crate::parse_vhdl_entities::parse_vhdl_entities;
    use super::fixtures::*;
    use super::testctx::test_context;

    fn src(text: &str) -> HdlSource {
        HdlSource { text: text.to_string(), language: String::new() }
    }

    // Golden: every field hand-checked against the fixture text (an oracle
    // independent of this package's own code — read directly against
    // IEEE 1076 entity/port/generic syntax), including a standalone entity
    // (no legacy component-mirror needed).
    #[test]
    fn test_entity_golden_shape() {
        let ax = test_context();
        let out = parse_vhdl_entities(&ax, src(VHDL_COUNTER)).unwrap();
        assert_eq!(out.error, "");
        assert!(!out.has_error);
        assert_eq!(out.entity_count, 1);
        let e = &out.entities[0];
        assert_eq!(e.name, "counter");
        assert_eq!(e.kind, "entity");
        assert!(!e.has_error);
        assert_eq!(e.doc, "A simple up counter with synchronous reset.");

        assert_eq!(e.parameters.len(), 2);
        assert_eq!(e.parameters[0].name, "WIDTH");
        assert_eq!(e.parameters[0].data_type, "integer");
        assert_eq!(e.parameters[0].default_value, "8");
        assert_eq!(e.parameters[1].name, "STEP");
        assert_eq!(e.parameters[1].data_type, "integer");
        assert_eq!(e.parameters[1].default_value, "1");

        assert_eq!(e.ports.len(), 3);
        assert_eq!(e.ports[0].name, "clk");
        assert_eq!(e.ports[0].direction, "input");
        assert_eq!(e.ports[0].data_type, "std_logic");
        assert_eq!(e.ports[0].width, "");
        assert_eq!(e.ports[0].doc, "system clock");
        assert_eq!(e.ports[1].name, "rst_n");
        assert_eq!(e.ports[1].doc, "active-low reset");
        assert_eq!(e.ports[2].name, "count");
        assert_eq!(e.ports[2].direction, "output");
        assert_eq!(e.ports[2].data_type, "std_logic_vector");
        assert_eq!(e.ports[2].width, "(WIDTH-1 downto 0)");
    }

    // Malformed source is reported structurally, not as a top-level error.
    #[test]
    fn test_malformed_source_sets_has_error_not_error() {
        let ax = test_context();
        let out = parse_vhdl_entities(&ax, src(VHDL_MALFORMED)).unwrap();
        assert_eq!(out.error, "");
        assert!(out.has_error);
        assert_eq!(out.entity_count, 0);
    }

    // Regression (found by adversarial review): the entity IS found (its
    // name parses cleanly) but its port list contains a syntax error — the
    // per-entity has_error must be true even though the file-level parse
    // otherwise "succeeds" (an entity was recovered), so a caller can tell
    // the extracted ports (direction flipped, one dropped) are not reliable.
    #[test]
    fn test_entity_with_malformed_port_list_flags_entity_has_error() {
        let ax = test_context();
        let out = parse_vhdl_entities(&ax, src(VHDL_PORT_LIST_MISSING_SEMICOLON)).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.entity_count, 1);
        assert_eq!(out.entities[0].name, "top");
        assert!(out.entities[0].has_error);
    }

    #[test]
    fn test_large_input_no_crash() {
        // Payload-size limits are the platform's job, not this node's; a
        // large input must still parse cleanly (recovering zero entities
        // from non-HDL text) instead of crashing.
        let ax = test_context();
        let big = "x".repeat(2_000_001);
        let out = parse_vhdl_entities(&ax, src(&big)).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.entity_count, 0);
    }

    #[test]
    fn test_deterministic() {
        let ax = test_context();
        let a = parse_vhdl_entities(&ax, src(VHDL_COUNTER)).unwrap();
        let b = parse_vhdl_entities(&ax, src(VHDL_COUNTER)).unwrap();
        assert_eq!(a.entities[0].name, b.entities[0].name);
        assert_eq!(a.entities[0].ports.len(), b.entities[0].ports.len());
    }
}
