// Tests for the ListDefinitions node.
#[cfg(test)]
#[path = "testctx.rs"]
mod testctx;
#[cfg(test)]
#[path = "fixtures.rs"]
mod fixtures;

#[cfg(test)]
mod tests {
    use crate::gen::messages::HdlSource;
    use crate::list_definitions::list_definitions;
    use super::fixtures::*;
    use super::testctx::test_context;

    fn src(text: &str, language: &str) -> HdlSource {
        HdlSource { text: text.to_string(), language: language.to_string() }
    }

    #[test]
    fn test_verilog_autodetect() {
        let ax = test_context();
        let out = list_definitions(&ax, src(VERILOG_TWO_MODULES, "")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.language, "verilog");
        assert_eq!(out.count, 2);
        assert_eq!(out.names, vec!["first_mod", "second_mod"]);
    }

    #[test]
    fn test_vhdl_autodetect() {
        let ax = test_context();
        let out = list_definitions(&ax, src(VHDL_COUNTER, "")).unwrap();
        assert_eq!(out.language, "vhdl");
        assert_eq!(out.count, 1);
        assert_eq!(out.names, vec!["counter"]);
    }

    // An explicit hint is honored even when it looks unusual — it always
    // wins over auto-detection.
    #[test]
    fn test_explicit_hint_is_honored() {
        let ax = test_context();
        let out = list_definitions(&ax, src(VERILOG_COUNTER, "SystemVerilog")).unwrap();
        assert_eq!(out.language, "verilog");
        assert_eq!(out.names, vec!["counter"]);
    }

    #[test]
    fn test_ambiguous_input_returns_empty_not_error() {
        let ax = test_context();
        let out = list_definitions(&ax, src("", "")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.language, "");
        assert_eq!(out.count, 0);
        assert!(out.names.is_empty());
    }
}
