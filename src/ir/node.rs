use std::collections::HashMap;
use std::rc::Weak;

use base::SourceLocation;

pub enum NodeType {
    Operation,
    Integer,
}

//TODO: remove this?
pub enum ProtoNode<'a, Value: 'static> {
    // TODO: use some kind of BigInt?
    Integer {value: usize},
    // TODO: support strings.
    Operation {
        operation: &'static Operation<Value>,
        operands: &'a Iterator<Item = ProtoNode<'a, Value>>,
    },
}

pub enum EvaluationResult<Value: 'static> {
    Complete(Value),
    Incomplete
}

// TODO: unify this with EvaluationResult?
// TODO: rename?
pub struct Incomplete<Value: 'static> {
    operation: &'static Operation<Value>,
    parent: Weak<Incomplete<Value>>,
    operands: Vec<EvaluationResult<Value>>,
}

impl<Value> Incomplete<Value> {
    //TODO: create a function that produces an Iterator<Item=Value>, but only when all arguments
    // are complete
}

// TODO: remove this alias if we don't end up needing it.
pub type Evaluator<V: 'static> = fn(&mut Iterator<Item = V>) -> EvaluationResult<V>;

// TODO: create an Arguments type? (useful for eager ops)
// TODO: handle lazy ops. (Consuming a stream of ProtoNodes *might* be a workable solution).
pub struct Operation<Value: 'static> {

    name: &'static str,
    evaluator: Evaluator<Value>,
}

impl<Value: 'static> Operation<Value> {
    pub const fn new(name: &'static str, evaluator: Evaluator<Value>) -> Operation<Value> {
        Operation { name: name, evaluator: evaluator }
    }
}

pub struct OperationGroup<Value: 'static> {
    map: HashMap<&'static str, Operation<Value>>,
}

impl<Value> OperationGroup<Value> {
    pub fn new() -> OperationGroup<Value> {
        OperationGroup { map: HashMap::<&str, Operation<Value>>::new() }
    }

    pub fn insert(&mut self, op: Operation<Value>) {
        self.map.insert(op.name, op);
    }

    pub fn get(&self, name: &str) -> Option<&Operation<Value>> {
        self.map.get(name)
    }
}

trait Node<Value> {
    fn location(&self) -> SourceLocation;
    fn node_type(&self) -> NodeType;
    fn value(&self) -> Value;
}

struct OperationNode<Value: 'static> {
    operation: &'static Operation<Value>,
}
