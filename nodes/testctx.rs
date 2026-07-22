// Shared test-only mock AxiomContext for hdl-tools node tests. Included by
// each `nodes/<node>_test.rs` via `#[path = "testctx.rs"] mod testctx;`
// inside its `#[cfg(test)] mod tests` block. Compiled once per including test
// module (like hdlutil.rs for the nodes), so there is no cross-module
// collision.
#![allow(dead_code)]

use crate::axiom_context::*;
use std::collections::HashMap;

struct TestLogger;
impl AxiomLogger for TestLogger {
    fn debug(&self, _m: &str, _a: &HashMap<&str, String>) {}
    fn info(&self, _m: &str, _a: &HashMap<&str, String>) {}
    fn warn(&self, _m: &str, _a: &HashMap<&str, String>) {}
    fn error(&self, _m: &str, _a: &HashMap<&str, String>) {}
}
struct TestSecrets;
impl AxiomSecrets for TestSecrets {
    fn get(&self, _n: &str) -> (String, bool) {
        (String::new(), false)
    }
}
struct EmptyFlow {
    pos: FlowPosition,
}
impl FlowReflection for EmptyFlow {
    fn nodes(&self) -> &[ReflectionNode] {
        &[]
    }
    fn edges(&self) -> &[ReflectionEdge] {
        &[]
    }
    fn loop_edges(&self) -> &[ReflectionEdge] {
        &[]
    }
    fn position(&self) -> &FlowPosition {
        &self.pos
    }
    fn graph_id(&self) -> &str {
        ""
    }
}
struct TestReflection {
    flow: EmptyFlow,
}
impl Reflection for TestReflection {
    fn flow(&self) -> &dyn FlowReflection {
        &self.flow
    }
}
struct TestFlowMut;
impl FlowMutation for TestFlowMut {
    fn add_node(&self, _p: &str, _v: &str, _c: Option<CanvasPosition>) -> u32 {
        0
    }
    fn add_edge(&self, _s: u32, _d: u32, _c: Option<EdgeCondition>) {}
}
struct TestMutation {
    flow: TestFlowMut,
}
impl Mutation for TestMutation {
    fn flow(&self) -> &dyn FlowMutation {
        &self.flow
    }
}
pub struct TestContext {
    log: TestLogger,
    secrets: TestSecrets,
    reflection: TestReflection,
    mutation: TestMutation,
}
impl AxiomContext for TestContext {
    fn log(&self) -> &dyn AxiomLogger {
        &self.log
    }
    fn secrets(&self) -> &dyn AxiomSecrets {
        &self.secrets
    }
    fn execution_id(&self) -> &str {
        "test-execution-id"
    }
    fn flow_id(&self) -> &str {
        "test-flow-id"
    }
    fn tenant_id(&self) -> &str {
        "test-tenant-id"
    }
    fn reflection(&self) -> &dyn Reflection {
        &self.reflection
    }
    fn mutation(&self) -> &dyn Mutation {
        &self.mutation
    }
}
pub fn test_context() -> TestContext {
    TestContext {
        log: TestLogger,
        secrets: TestSecrets,
        reflection: TestReflection {
            flow: EmptyFlow { pos: FlowPosition::default() },
        },
        mutation: TestMutation { flow: TestFlowMut },
    }
}
