// Tests for the DetectLanguage node. Wired by the generated service as
// `#[cfg(test)] #[path="nodes/detect_language_test.rs"] mod detect_language_test;`.
#[cfg(test)]
#[path = "testctx.rs"]
mod testctx;
#[cfg(test)]
#[path = "fixtures.rs"]
mod fixtures;

#[cfg(test)]
mod tests {
    use crate::detect_language::detect_language;
    use crate::gen::messages::HdlSource;
    use super::fixtures::*;
    use super::testctx::test_context;

    fn src(text: &str) -> HdlSource {
        HdlSource { text: text.to_string(), language: String::new() }
    }

    // Independent oracle: cross-parsing each fixture with the OTHER grammar
    // produces syntax errors (verified directly with a standalone tree-sitter
    // harness outside this package's code) while the native grammar parses
    // clean, so confidence must be the maximum 1.0.
    #[test]
    fn test_detects_verilog_with_full_confidence() {
        let ax = test_context();
        let out = detect_language(&ax, src(VERILOG_COUNTER)).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.language, "verilog");
        assert_eq!(out.verilog_error_nodes, 0);
        assert!(out.vhdl_error_nodes > 0);
        assert_eq!(out.confidence, 1.0);
    }

    #[test]
    fn test_detects_vhdl_with_full_confidence() {
        let ax = test_context();
        let out = detect_language(&ax, src(VHDL_COUNTER)).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.language, "vhdl");
        assert_eq!(out.vhdl_error_nodes, 0);
        assert!(out.verilog_error_nodes > 0);
        assert_eq!(out.confidence, 1.0);
    }

    #[test]
    fn test_nonansi_verilog_detected() {
        let ax = test_context();
        let out = detect_language(&ax, src(VERILOG_NONANSI)).unwrap();
        assert_eq!(out.language, "verilog");
    }

    // A trivial/empty input parses cleanly under BOTH grammars — genuinely
    // ambiguous, so language must be "" rather than an arbitrary pick.
    #[test]
    fn test_empty_input_is_ambiguous() {
        let ax = test_context();
        let out = detect_language(&ax, src("")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.language, "");
        assert_eq!(out.verilog_error_nodes, 0);
        assert_eq!(out.vhdl_error_nodes, 0);
    }

    #[test]
    fn test_oversized_input_is_input_too_large() {
        let ax = test_context();
        let big = "x".repeat(2_000_001);
        let out = detect_language(&ax, src(&big)).unwrap();
        assert_eq!(out.error, "INPUT_TOO_LARGE");
        assert_eq!(out.language, "");
    }

    // Determinism: identical input yields an identical verdict.
    #[test]
    fn test_deterministic() {
        let ax = test_context();
        let a = detect_language(&ax, src(VERILOG_COUNTER)).unwrap();
        let b = detect_language(&ax, src(VERILOG_COUNTER)).unwrap();
        assert_eq!(a.language, b.language);
        assert_eq!(a.confidence, b.confidence);
        assert_eq!(a.verilog_error_nodes, b.verilog_error_nodes);
        assert_eq!(a.vhdl_error_nodes, b.vhdl_error_nodes);
    }
}
