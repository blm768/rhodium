use std::collections::HashMap;

use base::SourceLocation;

pub enum NodeType {
    Operation,
    Integer,
}

pub enum EvaluationResult<T: 'static> {
    Complete { value: Box<T> },
    Incomplete
}

// TODO: create an Arguments type? (useful for eager ops)
// TODO: handle eager vs. lazy ops.
pub struct Operation<Value: 'static> {
    name: &'static str,
    evaluate: &'static fn(&Node) -> EvaluationResult<Value>,
}

pub struct OperationGroup<Value: 'static> {
    map: HashMap<&'static str, Value>,
}

impl<Value> OperationGroup<Value> {
    fn get(&self, name: &str) -> Option<&'static Operation<Value>> {
        None
    }
}

// TODO: make this generic on value types, too.
trait Node {
    fn location(&self) -> SourceLocation;
    fn node_type(&self) -> NodeType;
}

struct OperationNode<Value: 'static> {
    operation: &'static Operation<Value>,
}
