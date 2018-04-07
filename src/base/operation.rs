use std;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::rc::{Rc, Weak};

/**
 * A trait for all values that can be handled by Operation
 */
// TODO: fold this into EvaluationContext?
pub trait Value: Clone + Debug {}

// TODO: remove Clone once it's not required for #[derive(Clone)] on Operation?
pub trait EvaluationContext: Clone {
    type Value: Value + 'static;
}

/**
 * Responds to expressions becoming total
 */
pub trait EvaluationListener<C: EvaluationContext>: Debug {
    fn on_evaluated(&self, partial: &PartialExpression<C>, value: C::Value);
}

/**
 * A list of operands
 */
#[derive(Debug)]
pub enum OperandList<C: EvaluationContext> {
    Total(Vec<C::Value>),
    Partial(Vec<Expression<C>>, usize),
}

impl<C: EvaluationContext> OperandList<C> {
    pub fn new(operands: Vec<Expression<C>>) -> OperandList<C> {
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
pub struct PartialExpression<C: EvaluationContext> {
    operation: Operation<C>,
    context: C,
    operands: RefCell<OperandList<C>>,
    listener: Cell<Option<Weak<EvaluationListener<C>>>>,
    /**
     * The operand's index in the parent expression
     *
     * Will be set when the PartialExpression is placed in an OperandList
     */
    index: Cell<usize>,
}

// Since we can't derive Debug for Cell<Option<Weak<...>>>, we have to implement it manually.
impl<C: EvaluationContext> Debug for PartialExpression<C> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        // TODO: implement properly.
        write!(f, "PartialExpression")
    }
}

impl<C: EvaluationContext + 'static> PartialExpression<C> {
    pub fn new(op: Operation<C>, context: C, operands: OperandList<C>) -> Rc<PartialExpression<C>> {
        let exp = Rc::new(PartialExpression {
            operation: op,
            context: context,
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
                            .set(Some(Rc::<PartialExpression<C>>::downgrade(&exp)));
                    }
                }
            }
        }

        exp
    }
}

impl<C: EvaluationContext> EvaluationListener<C> for PartialExpression<C> {
    fn on_evaluated(&self, partial: &PartialExpression<C>, value: C::Value) {
        let operand_list = &mut *self.operands.borrow_mut();

        let mut eval_result = EvaluationResult::Pending;
        // If the OperandList becomes total, the new total list will be stored here.
        let mut new_operands: Option<OperandList<C>> = None;

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

                    eval_result = self.operation.evaluate(&self.context, &total_operands);
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
pub enum Expression<C: EvaluationContext> {
    /// An expression that has been totally evaluated
    Total(C::Value),
    /// An expression which has not been totally evaluated
    Partial(Rc<PartialExpression<C>>),
}

impl<C: EvaluationContext + 'static> Expression<C> {
    /**
     * Builds an Expression from an Operation and a list of Operands
     */
    pub fn from_op(op: Operation<C>, context: &C, operands: Vec<Expression<C>>) -> Expression<C> {
        let op_list = OperandList::new(operands);

        if let OperandList::Total(operand_values) = op_list {
            let eval_result = op.evaluate(&context, &operand_values);

            return match eval_result {
                EvaluationResult::Total(val) => Expression::Total(val),
                EvaluationResult::Pending => {
                    let partial = PartialExpression::new(
                        op,
                        context.clone(),
                        OperandList::Total(operand_values),
                    );
                    Expression::Partial(partial)
                }
            };
        }

        Expression::Partial(PartialExpression::new(op, context.clone(), op_list))
    }

    /**
     * Builds a total Expression from a value
     */
    pub fn from_value(value: C::Value) -> Expression<C> {
        Expression::Total(value)
    }
}

pub enum EvaluationResult<V: Value + 'static> {
    Total(V),
    Pending,
}

// TODO: handle "lazy"/quoted ops? (Consuming a stream of ProtoNodes *might* be a workable solution).
#[derive(Clone)]
pub struct Operation<C: EvaluationContext> {
    name: &'static str,
    evaluator: fn(&C, &[C::Value]) -> EvaluationResult<C::Value>,
}

impl<C: EvaluationContext> Operation<C> {
    pub const fn new(
        name: &'static str,
        evaluator: fn(&C, &[C::Value]) -> EvaluationResult<C::Value>,
    ) -> Operation<C> {
        Operation {
            name: name,
            evaluator: evaluator,
        }
    }

    pub fn evaluate(&self, context: &C, operands: &[C::Value]) -> EvaluationResult<C::Value> {
        (self.evaluator)(context, operands)
    }
}

// For some reason I don't understand, #[derive(Copy)] seems to fail silently, so we have
// to mark this type manually.
impl<C: EvaluationContext> Copy for Operation<C> {}

impl<C: EvaluationContext> Debug for Operation<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Operation {{ name: {} }}", self.name)
    }
}

pub struct OperationGroup<C: EvaluationContext> {
    map: HashMap<Box<str>, Operation<C>>,
}

impl<C: EvaluationContext> OperationGroup<C> {
    pub const fn new(map: HashMap<Box<str>, Operation<C>>) -> OperationGroup<C> {
        OperationGroup { map: map }
    }

    pub fn get(&self, name: &str) -> Option<&Operation<C>> {
        self.map.get(name)
    }
}
