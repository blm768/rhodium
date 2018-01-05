use base::SourceLocation;

use base::operation;
use base::operation::Operation;

pub enum NodeType {
    Operation,
    Integer,
}

//TODO: remove this?
pub enum ProtoNode<'a, Value: operation::Value + 'static> {
    // TODO: use some kind of BigInt?
    Integer { value: usize },
    // TODO: support strings.
    Operation {
        operation: &'static Operation<Value>,
        operands: &'a Iterator<Item = ProtoNode<'a, Value>>,
    },
}

trait Node<Value> {
    fn location(&self) -> SourceLocation;
    fn node_type(&self) -> NodeType;
    fn value(&self) -> Value;
}

struct OperationNode<Value: operation::Value + 'static> {
    operation: &'static Operation<Value>,
}
