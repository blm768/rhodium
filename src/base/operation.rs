use std::collections::HashMap;
use std::rc::Rc;
use std::rc::Weak;

use phf;

// TODO: use for stuff below?
pub trait Value {

}

pub enum Expression<V: 'static> {
    Complete(V),
    Incomplete {
        operation: Operation<V>,
        parent: Option<Weak<Expression<V>>>,
        operands: Vec<Rc<Expression<V>>>,
        num_incomplete: usize,
    }
}

impl<V> Expression<V> {
    pub fn new_incomplete(op: Operation<V>, operands: Vec<Rc<Expression<V>>>) -> Rc<Expression<V>> {
        let mut num_incomplete = 0;

        // TODO: set childrens' parents.
        // TODO: set num_incomplete.
        let expression = Rc::new(Expression::Incomplete {
            operation: op,
            parent: None,
            operands: operands,
            num_incomplete: 0
        });

        // TODO: implement.
        /*for child in operands {
            if let Expression::Incomplete {ref parent} = child {
                num_incomplete += 1;
                parent = expression;
            }
        }*/

        expression
    }

    pub fn new_complete(value: V) -> Rc<Expression<V>> {
        Rc::new(Expression::Complete(value))
    }


    // Called when an operand becomes complete
    // TODO: implement.
    fn on_complete(&mut self) {}
    //TODO: create a function that produces an Iterator<Item=V>, but only when all arguments
    // are complete
}

pub enum EvaluationResult<V: 'static> {
    Complete(V),
    Incomplete
}

pub type Evaluator<V: 'static> = fn(&mut Iterator<Item = V>) -> EvaluationResult<V>;

// TODO: create an Arguments type? (useful for eager ops)
// TODO: handle lazy ops. (Consuming a stream of ProtoNodes *might* be a workable solution).
#[derive(Copy)]
pub struct Operation<V: 'static> {

    name: &'static str,
    evaluator: Evaluator<V>,
}

impl<V: 'static> Operation<V> {
    pub const fn new(name: &'static str, evaluator: Evaluator<V>) -> Operation<V> {
        Operation { name: name, evaluator: evaluator }
    }
}

// Manual implementation required to get around Rust bug #28229
impl<V: 'static> Clone for Operation<V> {
    fn clone(&self) -> Operation<V> {
        Operation {
            name: self.name,
            evaluator: self.evaluator
        }
    }
}

pub struct OperationGroup<V: 'static> {
    map: phf::Map<&'static str, Operation<V>>,
}

impl<V> OperationGroup<V> {
    pub const fn new(map: phf::Map<&'static str, Operation<V>>) -> OperationGroup<V> {
        OperationGroup { map: map }
    }

    pub fn get(&self, name: &str) -> Option<&Operation<V>> {
        self.map.get(name)
    }
}
