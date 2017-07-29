use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::rc::Weak;

use phf;

pub trait Value: Clone {

}

// Represents the "meat" of an incomplete expression
pub struct IncompleteOp<V: Value + 'static> {
    operation: Operation<V>,
    operands: Vec<Rc<Expression<V>>>,
    parent_expression: Cell<Option<Weak<Expression<V>>>>,
    num_incomplete: usize,
}

pub struct PendingOp<V: Value + 'static> {
    operation: Operation<V>,
    operands: Vec<V>,
    parent_expression: Cell<Option<Weak<Expression<V>>>>,
}

pub enum Expression<V: Value + 'static> {
    Complete(V),
    Pending(PendingOp<V>),
    Incomplete(IncompleteOp<V>),
}

impl<V: Value + 'static> Expression<V> {
    pub fn from_op(op: Operation<V>, mut operands: Vec<Rc<Expression<V>>>) -> Rc<Expression<V>> {
        let mut num_incomplete = operands.iter().filter(|child| {
            match *Rc::as_ref(child) {
                Expression::Complete(_) => false,
                _ => true
            }
        }).count();

        if num_incomplete == 0 {
            let operand_values = operands.iter().map(|o| {
                match *Rc::as_ref(o) {
                    Expression::Complete(ref value) => value.clone(),
                    _ => unreachable!()
                }
            }).collect();

            // TODO: try to evaluate the op here.
            return Rc::new(Expression::Pending(
                PendingOp {
                    operation: op,
                    operands: operand_values,
                    parent_expression: Cell::new(None),
                }
            ))
        }

        // TODO: set childrens' parents.
        // TODO: handle the case when all arguments are complete and evaluate right away.
        let mut expression = Rc::new(Expression::Incomplete(
            IncompleteOp {
                operation: op,
                operands: vec!(),
                parent_expression: Cell::new(None),
                num_incomplete: num_incomplete
            }
        ));

        // TODO: implement.
        for child in operands.iter_mut() {
            if let Expression::Incomplete(ref oi) = *Rc::as_ref(child) {
                oi.parent_expression.set(Some(Rc::downgrade(&expression)));
            }
        }

        match *Rc::get_mut(&mut expression).unwrap() {
            Expression::Incomplete(ref mut oi) => {
                oi.operands = operands;
            },
            _ => unreachable!()
        }

        expression
    }

    pub fn from_value(value: V) -> Rc<Expression<V>> {
        Rc::new(Expression::Complete(value))
    }

    // Called when an operand becomes complete
    // TODO: implement.
    fn on_child_complete(&mut self) {
        match *self {
            Expression::Complete(_) => panic!("Invalid call to on_child_complete"),
            Expression::Pending(_) => panic!("Invalid call to on_child_complete"),
            Expression::Incomplete(ref mut oi) => {
                assert!(oi.num_incomplete > 0);
                oi.num_incomplete -= 1;
                // TODO: evaluate if num_incomplete hits 0.
            }
        }
    }

    //TODO: create a function that produces an Iterator<Item=V>, but only when all arguments
    // are complete?
}

pub enum EvaluationResult<V: Value + 'static> {
    Complete(V),
    Pending
}

pub type Evaluator<V: Value + 'static> = fn(&mut Iterator<Item = V>) -> EvaluationResult<V>;

// TODO: create an Arguments type? (useful for eager ops)
// TODO: handle lazy ops. (Consuming a stream of ProtoNodes *might* be a workable solution).
#[derive(Copy)]
pub struct Operation<V: Value + 'static> {
    name: &'static str,
    evaluator: Evaluator<V>,
}

impl<V: Value + 'static> Operation<V> {
    pub const fn new(name: &'static str, evaluator: Evaluator<V>) -> Operation<V> {
        Operation { name: name, evaluator: evaluator }
    }

    pub fn evaluate(&self, iterator: &mut Iterator<Item = V>) -> EvaluationResult<V> {
        (self.evaluator)(iterator)
    }
}

// Manual implementation required to get around Rust bug #28229
impl<V: Value + 'static> Clone for Operation<V> {
    fn clone(&self) -> Operation<V> {
        Operation {
            name: self.name,
            evaluator: self.evaluator
        }
    }
}

pub struct OperationGroup<V: Value + 'static> {
    map: phf::Map<&'static str, Operation<V>>,
}

impl<V: Value + 'static> OperationGroup<V> {
    pub const fn new(map: phf::Map<&'static str, Operation<V>>) -> OperationGroup<V> {
        OperationGroup { map: map }
    }

    pub fn get(&self, name: &str) -> Option<&Operation<V>> {
        self.map.get(name)
    }
}
