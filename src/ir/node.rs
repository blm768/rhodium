use base::source::SourceLocation;

use base::operation::{EvaluationContext, Operation};

pub enum NodeType {
    Operation,
    Integer,
}

//TODO: remove this?
pub enum ProtoNode<'a, C: EvaluationContext + 'a> {
    // TODO: use some kind of BigInt?
    Integer {
        value: usize,
    },
    // TODO: support strings.
    Operation {
        operation: Operation<C>,
        operands: &'a Iterator<Item = ProtoNode<'a, C>>,
    },
}

trait Node<C: EvaluationContext> {
    fn location(&self) -> SourceLocation;
    fn node_type(&self) -> NodeType;
    fn value(&self) -> C::Value;
}

struct OperationNode<C: EvaluationContext> {
    operation: Operation<C>,
}
