// Tests for the ExtractParameters node.
#[cfg(test)]
#[path = "testctx.rs"]
mod testctx;
#[cfg(test)]
#[path = "fixtures.rs"]
mod fixtures;

#[cfg(test)]
mod tests {
    use crate::extract_parameters::extract_parameters;
    use crate::gen::messages::{HdlSource, NamedHdlQuery};
    use super::fixtures::*;
    use super::testctx::test_context;

    fn query(text: &str, name: &str) -> NamedHdlQuery {
        NamedHdlQuery {
            source: Some(HdlSource { text: text.to_string(), language: String::new() }),
            name: name.to_string(),
        }
    }

    #[test]
    fn test_verilog_parameters() {
        let ax = test_context();
        let out = extract_parameters(&ax, query(VERILOG_COUNTER, "counter")).unwrap();
        assert_eq!(out.error, "");
        assert!(out.found);
        assert_eq!(out.count, 2);
        assert_eq!(out.parameters[0].name, "WIDTH");
        assert_eq!(out.parameters[0].default_value, "8");
        assert_eq!(out.parameters[1].name, "STEP");
        assert_eq!(out.parameters[1].data_type, "integer");
        assert_eq!(out.parameters[1].default_value, "1");
    }

    #[test]
    fn test_vhdl_generics() {
        let ax = test_context();
        let out = extract_parameters(&ax, query(VHDL_COUNTER, "counter")).unwrap();
        assert_eq!(out.count, 2);
        assert_eq!(out.parameters[0].name, "WIDTH");
        assert_eq!(out.parameters[0].data_type, "integer");
        assert_eq!(out.parameters[0].default_value, "8");
    }

    // A module with no parameters returns an empty (not missing) list.
    #[test]
    fn test_no_parameters_is_empty_list() {
        let ax = test_context();
        let out = extract_parameters(&ax, query(VERILOG_NONANSI, "adder")).unwrap();
        assert!(out.found);
        assert_eq!(out.count, 0);
        assert!(out.parameters.is_empty());
    }
}
