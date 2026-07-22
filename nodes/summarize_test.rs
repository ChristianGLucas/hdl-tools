// Tests for the Summarize node.
#[cfg(test)]
#[path = "testctx.rs"]
mod testctx;
#[cfg(test)]
#[path = "fixtures.rs"]
mod fixtures;

#[cfg(test)]
mod tests {
    use crate::gen::messages::HdlSource;
    use crate::summarize::summarize;
    use super::fixtures::*;
    use super::testctx::test_context;

    fn src(text: &str) -> HdlSource {
        HdlSource { text: text.to_string(), language: String::new() }
    }

    #[test]
    fn test_verilog_summary() {
        let ax = test_context();
        let out = summarize(&ax, src(VERILOG_COUNTER)).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.language, "verilog");
        assert_eq!(out.module_count, 1);
        assert_eq!(out.entity_count, 0);
        assert_eq!(out.package_count, 0);
        assert_eq!(out.port_count, 3);
        assert_eq!(out.param_count, 2);
        assert!(!out.has_error);
    }

    #[test]
    fn test_vhdl_summary() {
        let ax = test_context();
        let out = summarize(&ax, src(VHDL_COUNTER)).unwrap();
        assert_eq!(out.language, "vhdl");
        assert_eq!(out.entity_count, 1);
        assert_eq!(out.module_count, 0);
        assert_eq!(out.port_count, 3);
        assert_eq!(out.param_count, 2);
    }

    #[test]
    fn test_vhdl_package_only_summary() {
        let ax = test_context();
        let out = summarize(&ax, src(VHDL_PACKAGE)).unwrap();
        assert_eq!(out.language, "vhdl");
        assert_eq!(out.entity_count, 0);
        assert_eq!(out.package_count, 1);
        assert_eq!(out.port_count, 0);
        assert_eq!(out.param_count, 0);
    }

    // Counts sum ACROSS multiple modules, not just the first.
    #[test]
    fn test_sums_across_multiple_modules() {
        let ax = test_context();
        let out = summarize(&ax, src(VERILOG_TWO_MODULES)).unwrap();
        assert_eq!(out.module_count, 2);
        assert_eq!(out.port_count, 4);
        assert_eq!(out.param_count, 0);
    }

    #[test]
    fn test_ambiguous_input_all_zero_not_error() {
        let ax = test_context();
        let out = summarize(&ax, src("")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.language, "");
        assert_eq!(out.module_count, 0);
        assert_eq!(out.entity_count, 0);
    }
}
