// Tests for the ParseVerilogModules node.
#[cfg(test)]
#[path = "testctx.rs"]
mod testctx;
#[cfg(test)]
#[path = "fixtures.rs"]
mod fixtures;

#[cfg(test)]
mod tests {
    use crate::gen::messages::HdlSource;
    use crate::parse_verilog_modules::parse_verilog_modules;
    use super::fixtures::*;
    use super::testctx::test_context;

    fn src(text: &str) -> HdlSource {
        HdlSource { text: text.to_string(), language: String::new() }
    }

    // Golden: every field of the ANSI module hand-checked against the fixture
    // text (an oracle independent of this package's own code — the fixture
    // was read directly against IEEE 1800-2017 module/port/parameter syntax).
    #[test]
    fn test_ansi_module_golden_shape() {
        let ax = test_context();
        let out = parse_verilog_modules(&ax, src(VERILOG_COUNTER)).unwrap();
        assert_eq!(out.error, "");
        assert!(!out.has_error);
        assert_eq!(out.module_count, 1);
        assert_eq!(out.modules.len(), 1);
        let m = &out.modules[0];
        assert_eq!(m.name, "counter");
        assert_eq!(m.kind, "module");
        assert_eq!(m.doc, "A simple up counter with synchronous reset.");

        assert_eq!(m.parameters.len(), 2);
        assert_eq!(m.parameters[0].name, "WIDTH");
        assert_eq!(m.parameters[0].data_type, "");
        assert_eq!(m.parameters[0].default_value, "8");
        assert_eq!(m.parameters[1].name, "STEP");
        assert_eq!(m.parameters[1].data_type, "integer");
        assert_eq!(m.parameters[1].default_value, "1");

        assert_eq!(m.ports.len(), 3);
        assert_eq!(m.ports[0].name, "clk");
        assert_eq!(m.ports[0].direction, "input");
        assert_eq!(m.ports[0].data_type, "wire");
        assert_eq!(m.ports[0].width, "");
        assert_eq!(m.ports[0].doc, "system clock");
        assert_eq!(m.ports[1].name, "rst_n");
        assert_eq!(m.ports[1].direction, "input");
        assert_eq!(m.ports[1].doc, "active-low reset");
        assert_eq!(m.ports[2].name, "count");
        assert_eq!(m.ports[2].direction, "output");
        assert_eq!(m.ports[2].data_type, "reg");
        assert_eq!(m.ports[2].width, "[WIDTH-1:0]");
        assert_eq!(m.ports[2].doc, "");
    }

    // Non-ANSI (classic) style resolves direction/type/width from the
    // separate body declarations, matched back to the header's port order.
    #[test]
    fn test_nonansi_module_resolves_body_declarations() {
        let ax = test_context();
        let out = parse_verilog_modules(&ax, src(VERILOG_NONANSI)).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.module_count, 1);
        let m = &out.modules[0];
        assert_eq!(m.name, "adder");
        assert_eq!(m.ports.len(), 3);
        assert_eq!(m.ports[0].name, "a");
        assert_eq!(m.ports[0].direction, "input");
        assert_eq!(m.ports[0].width, "[3:0]");
        assert_eq!(m.ports[1].name, "b");
        assert_eq!(m.ports[1].direction, "input");
        assert_eq!(m.ports[1].width, "[3:0]");
        assert_eq!(m.ports[2].name, "sum");
        assert_eq!(m.ports[2].direction, "output");
        assert_eq!(m.ports[2].width, "[4:0]");
    }

    // Multiple modules in one file are all found, in source order.
    #[test]
    fn test_multiple_modules() {
        let ax = test_context();
        let out = parse_verilog_modules(&ax, src(VERILOG_TWO_MODULES)).unwrap();
        assert_eq!(out.module_count, 2);
        assert_eq!(out.modules[0].name, "first_mod");
        assert_eq!(out.modules[1].name, "second_mod");
        // A bare "input a" (no explicit net type keyword) has an empty type.
        assert_eq!(out.modules[0].ports[0].name, "a");
        assert_eq!(out.modules[0].ports[0].direction, "input");
        assert_eq!(out.modules[0].ports[0].data_type, "");
    }

    // Malformed source is NOT a top-level error — it is reported structurally
    // (has_error=true, best-effort/empty modules), matching the same
    // has_error/error split code-parse-tools uses.
    #[test]
    fn test_malformed_source_sets_has_error_not_error() {
        let ax = test_context();
        let out = parse_verilog_modules(&ax, src(VERILOG_MALFORMED)).unwrap();
        assert_eq!(out.error, "");
        assert!(out.has_error);
        assert_eq!(out.module_count, 0);
    }

    #[test]
    fn test_oversized_input_is_input_too_large() {
        let ax = test_context();
        let big = "x".repeat(2_000_001);
        let out = parse_verilog_modules(&ax, src(&big)).unwrap();
        assert_eq!(out.error, "INPUT_TOO_LARGE");
        assert_eq!(out.module_count, 0);
    }

    #[test]
    fn test_deterministic() {
        let ax = test_context();
        let a = parse_verilog_modules(&ax, src(VERILOG_COUNTER)).unwrap();
        let b = parse_verilog_modules(&ax, src(VERILOG_COUNTER)).unwrap();
        assert_eq!(a.modules.len(), b.modules.len());
        assert_eq!(a.modules[0].name, b.modules[0].name);
        assert_eq!(a.modules[0].ports.len(), b.modules[0].ports.len());
    }
}
