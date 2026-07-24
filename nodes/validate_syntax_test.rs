// Tests for the ValidateSyntax node.
#[cfg(test)]
#[path = "testctx.rs"]
mod testctx;
#[cfg(test)]
#[path = "fixtures.rs"]
mod fixtures;

#[cfg(test)]
mod tests {
    use crate::gen::messages::HdlSource;
    use crate::validate_syntax::validate_syntax;
    use super::fixtures::*;
    use super::testctx::test_context;

    fn src(text: &str, language: &str) -> HdlSource {
        HdlSource { text: text.to_string(), language: language.to_string() }
    }

    #[test]
    fn test_clean_verilog_is_ok() {
        let ax = test_context();
        let out = validate_syntax(&ax, src(VERILOG_COUNTER, "verilog")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.language, "verilog");
        assert!(out.ok);
        assert_eq!(out.issue_count, 0);
        assert!(out.issues.is_empty());
    }

    #[test]
    fn test_clean_vhdl_is_ok() {
        let ax = test_context();
        let out = validate_syntax(&ax, src(VHDL_COUNTER, "vhdl")).unwrap();
        assert!(out.ok);
        assert_eq!(out.issue_count, 0);
    }

    // Malformed source is never a top-level `error` — it is reported as a
    // structured, non-empty issue list with ok=false. Every issue is a real,
    // in-bounds ERROR/MISSING region (verified against the source length),
    // not a crash and not a silently-wrong "looks fine" verdict.
    #[test]
    fn test_malformed_verilog_reports_issues() {
        let ax = test_context();
        let out = validate_syntax(&ax, src(VERILOG_MALFORMED, "verilog")).unwrap();
        assert_eq!(out.error, "");
        assert!(!out.ok);
        assert!(out.issue_count > 0);
        assert_eq!(out.issue_count as usize, out.issues.len());
        for issue in &out.issues {
            assert!(issue.kind == "ERROR" || issue.kind == "MISSING");
            assert!(issue.start_byte <= issue.end_byte);
            assert!((issue.end_byte as usize) <= VERILOG_MALFORMED.len());
        }
    }

    #[test]
    fn test_malformed_vhdl_reports_issues() {
        let ax = test_context();
        let out = validate_syntax(&ax, src(VHDL_MALFORMED, "vhdl")).unwrap();
        assert!(!out.ok);
        assert!(out.issue_count > 0);
    }

    #[test]
    fn test_large_input_no_crash() {
        // Payload-size limits are the platform's job, not this node's; a
        // large input that isn't valid HDL must still report structured
        // issues instead of crashing.
        let ax = test_context();
        let big = "x".repeat(2_000_001);
        let out = validate_syntax(&ax, src(&big, "verilog")).unwrap();
        assert_eq!(out.error, "");
        assert!(!out.ok);
        assert!(out.issue_count > 0);
    }
}
