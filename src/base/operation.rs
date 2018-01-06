use std;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::rc::{Rc, Weak};

/**
 * A trait for all values that can be handled by Operation
 */
pub trait Value: Clone + Debug {}

// TODO: use this.
pub trait ExpressionContext<V: Value + 'static> {}

/**
 * Responds to expressions becoming total
 */
pub trait EvaluationListener<V: Value + 'static>: Debug {
    fn on_evaluated(&self, partial: &PartialExpression<V>, value: V);
}

/**
 * A list of operands
 */
#[derive(Debug)]
pub enum OperandList<V: Value + 'static> {
    Total(Vec<V>),
    Partial(Vec<Expression<V>>, usize),
}

impl<V: Value + 'static> OperandList<V> {
    pub fn new(operands: Vec<Expression<V>>) -> OperandList<V> {
        let num_partial = operands
            .iter()
            .filter(|operand| match **operand {
                Expression::Total(_) => false,
                Expression::Partial(_) => true,
            })
            .count();

        if num_partial == 0 {
            OperandList::Total(
                operands
                    .into_iter()
                    .map(|operand| match operand {
                        Expression::Total(ref value) => value.clone(),
                        Expression::Partial(_) => unreachable!(),
                    })
                    .collect::<Vec<_>>(),
            )
        } else {
            // Set children's indices
            for (index, operand) in operands.iter().enumerate() {
                if let Expression::Partial(ref partial) = *operand {
                    partial.index.set(index);
                }
            }

            OperandList::Partial(operands, num_partial)
        }
    }
}

/**
 * Represents the "meat" of a partially-evaluated expression
 */
pub struct PartialExpression<V: Value + 'static> {
    operation: Operation<V>,
    operands: RefCell<OperandList<V>>,
    listener: Cell<Option<Weak<EvaluationListener<V>>>>,
    /**
     * The operand's index in the parent expression
     *
     * Will be set when the PartialExpression is placed in an OperandList
     */
    index: Cell<usize>,
}

// Since we can't derive Debug for Cell<Option<Weak<...>>>, we have to implement it manually.
impl<V: Value + 'static> Debug for PartialExpression<V> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        // TODO: implement properly.
        write!(f, "PartialExpression")
    }
}

impl<V: Value + 'static> PartialExpression<V> {
    pub fn new(op: Operation<V>, operands: OperandList<V>) -> Rc<PartialExpression<V>> {
        let exp = Rc::new(PartialExpression {
            operation: op,
            operands: RefCell::new(operands),
            listener: Cell::new(None),
            index: Cell::new(0),
        });

        // Set operands' listeners.
        {
            let operands = &*exp.operands.borrow();
            if let OperandList::Partial(ref operands, _) = *operands {
                for operand in operands.iter() {
                    if let Expression::Partial(ref oi) = *operand {
                        oi.listener
                            .set(Some(Rc::<PartialExpression<V>>::downgrade(&exp)));
                    }
                }
            }
        }

        exp
    }
}

impl<V: Value + 'static> EvaluationListener<V> for PartialExpression<V> {
    fn on_evaluated(&self, partial: &PartialExpression<V>, value: V) {
        let operand_list = &mut *self.operands.borrow_mut();

        let mut eval_result = EvaluationResult::Pending;
        // If the OperandList becomes total, the new total list will be stored here.
        let mut new_operands: Option<OperandList<V>> = None;

        if let OperandList::Partial(ref mut operands, ref mut num_partial) = *operand_list {
            if let Expression::Partial(_) = operands[partial.index.get()] {
                operands[partial.index.get()] = Expression::Total(value);
                *num_partial -= 1;

                // Are we ready to evaluate?
                if *num_partial == 0 {
                    let total_operands = operands
                        .iter()
                        .map(|o| match *o {
                            Expression::Total(ref value) => value.clone(),
                            _ => unreachable!(),
                        })
                        .collect::<Vec<_>>();

                    eval_result = self.operation.evaluate(&total_operands);
                    new_operands = Some(OperandList::Total(total_operands));
                }
            }
        }

        if let Some(operands) = new_operands {
            *operand_list = operands;
        }

        if let EvaluationResult::Total(value) = eval_result {
            // We know that there won't be a mutable borrow here because no one else should
            // be setting the listener.
            let maybe_listener = unsafe { &*self.listener.as_ptr() };
            if let Some(ref weak_listener) = *maybe_listener {
                if let Some(ref listener) = weak_listener.upgrade() {
                    listener.on_evaluated(self, value);
                }
            }
        }
    }
}

/**
 * An expression is either a value or an operation applied to a list of expressions.
 */
#[derive(Debug)]
pub enum Expression<V: Value + 'static> {
    /// An expression that has been totally evaluated
    Total(V),
    /// An expression which has not been totally evaluated
    Partial(Rc<PartialExpression<V>>),
}

impl<V: Value + 'static> Expression<V> {
    /**
     * Builds an Expression from an Operation and a list of Operands
     */
    pub fn from_op(op: Operation<V>, operands: Vec<Expression<V>>) -> Expression<V> {
        let op_list = OperandList::new(operands);

        if let OperandList::Total(operand_values) = op_list {
            let eval_result = op.evaluate(&operand_values);

            return match eval_result {
                EvaluationResult::Total(val) => Expression::Total(val),
                EvaluationResult::Pending => {
                    let partial = PartialExpression::new(op, OperandList::Total(operand_values));
                    Expression::Partial(partial)
                }
            };
        }

        Expression::Partial(PartialExpression::new(op, op_list))
    }

    /**
     * Builds a total Expression from a value
     */
    pub fn from_value(value: V) -> Expression<V> {
        Expression::Total(value)
    }
}

pub enum EvaluationResult<V: Value + 'static> {
    Total(V),
    Pending,
}

// TODO: handle lazy ops? (Consuming a stream of ProtoNodes *might* be a workable solution).
#[derive(Clone)]
pub struct Operation<V: Value + 'static> {
    name: &'static str,
    evaluator: fn(&[V]) -> EvaluationResult<V>,
}

// For some reason I don't understand, #[derive(Copy)] seems to fail silently, so we have
// to mark this type manually.
impl<V: Value + 'static> Copy for Operation<V> {}

impl<V: Value + 'static> Operation<V> {
    pub const fn new(
        name: &'static str,
        evaluator: fn(&[V]) -> EvaluationResult<V>,
    ) -> Operation<V> {
        Operation {
            name: name,
            evaluator: evaluator,
        }
    }

    pub fn evaluate(&self, operands: &[V]) -> EvaluationResult<V> {
        (self.evaluator)(operands)
    }
}

impl<V: Value + 'static> Debug for Operation<V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Operation {{ name: {} }}", self.name)
    }
}

pub struct OperationGroup<V: Value + 'static> {
    map: HashMap<Box<str>, Operation<V>>,
}

impl<V: Value + 'static> OperationGroup<V> {
    pub const fn new(map: HashMap<Box<str>, Operation<V>>) -> OperationGroup<V> {
        OperationGroup { map: map }
    }

    pub fn get(&self, name: &str) -> Option<&Operation<V>> {
        self.map.get(name)
    }
}
