use std::cell::Cell;
use std::fmt;
use std::fmt::Debug;
use std::rc::Rc;
use std::rc::Weak;

use phf;

pub trait Value: Clone + Debug {

}

// Represents the "meat" of an incomplete expression
pub struct IncompleteOp<V: Value + 'static> {
    operation: Operation<V>,
    operands: Vec<Rc<Expression<V>>>,
    parent_expression: Cell<Option<Weak<Expression<V>>>>,
    num_incomplete: usize,
}

impl<V: Value + 'static> Debug for IncompleteOp<V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "IncompleteOp({}, {:?})", self.operation.name, self.operands)
    }
}

pub struct PendingOp<V: Value + 'static> {
    operation: Operation<V>,
    operands: Vec<V>,
    parent_expression: Cell<Option<Weak<Expression<V>>>>,
}

impl<V: Value + 'static> Debug for PendingOp<V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PendingOp({}, {:?})", self.operation.name, self.operands)

    }
}

#[derive(Debug)]
pub enum Expression<V: Value + 'static> {
    Complete(V),
    Pending(PendingOp<V>),
    Incomplete(IncompleteOp<V>),
}

impl<V: Value + 'static> Expression<V> {
    pub fn from_op(op: Operation<V>, mut operands: Vec<Rc<Expression<V>>>) -> Rc<Expression<V>> {
        let num_incomplete = operands.iter().filter(|child| {
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
            }).collect::<Vec<_>>();

            let eval_result = op.evaluate(&operand_values);

            match eval_result {
                EvaluationResult::Complete(val) => {
                    return Rc::new(Expression::Complete(val));
                }
                EvaluationResult::Pending => {
                    return Rc::new(Expression::Pending(
                        PendingOp {
                            operation: op,
                            operands: operand_values,
                            parent_expression: Cell::new(None),
                        }
                    ));
                }
            }
        }

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

pub type Evaluator<V: Value + 'static> = fn(&Vec<V>) -> EvaluationResult<V>;

// TODO: create an Arguments type? (useful for eager ops)
// TODO: handle lazy ops. (Consuming a stream of ProtoNodes *might* be a workable solution).
#[derive(Copy)]
pub struct Operation<V: Value + 'static> {
    name: &'static str,
    evaluator: Evaluator<V>,
}

impl<V: Value + 'static> Debug for Operation<V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Operation {{ name: {} }}", self.name)
    }
}

impl<V: Value + 'static> Operation<V> {
    pub const fn new(name: &'static str, evaluator: Evaluator<V>) -> Operation<V> {
        Operation { name: name, evaluator: evaluator }
    }

    pub fn evaluate(&self, operands: &Vec<V>) -> EvaluationResult<V> {
        (self.evaluator)(operands)
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
