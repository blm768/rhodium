use std;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::mem;
use std::ptr;
use std::rc::{Rc, Weak};

/**
 * A trait for all values that can be handled by Operation
 */
pub trait Value: Clone + Debug {}

pub trait EvaluationContext: Clone + Debug {
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
    self_weak: Weak<PartialExpression<C>>,
    /**
     * The operand's index in the parent expression
     *
     * Will be set when the PartialExpression is placed in an OperandList
     */
    index: Cell<usize>,
}

impl<C: EvaluationContext + 'static> PartialExpression<C> {
    pub fn new(op: Operation<C>, context: C, operands: OperandList<C>) -> Rc<PartialExpression<C>> {
        let mut exp = Rc::new(PartialExpression {
            operation: op,
            context,
            operands: RefCell::new(operands),
            listener: Cell::new(None),
            self_weak: unsafe { mem::uninitialized() },
            index: Cell::new(0),
        });
        unsafe {
            ptr::write(
                &mut Rc::get_mut(&mut exp).unwrap().self_weak,
                Rc::<PartialExpression<C>>::downgrade(&exp),
            );
        }

        // Make exp listen for changes in its operands.
        {
            let operands = exp.operands.borrow();
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

    fn try_evaluate_self(&self) {
        let mut operands = self.operands.borrow_mut();

        let new_operands: Option<Vec<C::Value>> = match *operands {
            OperandList::Total(_) => None,
            OperandList::Partial(ref ops, num_partial) => {
                if num_partial == 0 {
                    Some(
                        ops.iter()
                            .map(|o| match *o {
                                Expression::Total(ref value) => value.clone(),
                                _ => unreachable!(),
                            })
                            .collect::<Vec<_>>(),
                    )
                } else {
                    None
                }
            }
        };

        if let Some(total_ops) = new_operands {
            *operands = OperandList::Total(total_ops);
        }

        if let OperandList::Total(ref total_ops) = *operands {
            let eval_result = self.operation.evaluate(&self.context, total_ops);
            match eval_result {
                EvaluationResult::Total(value) => self.on_self_evaluated(value),
                EvaluationResult::Pending => {
                    self.operation
                        .register(&self.context, &self.self_weak, total_ops);
                }
            }
        }
    }

    fn on_self_evaluated(&self, value: C::Value) {
        // We know that there won't be a mutable borrow of the listener here
        // because no one else will be setting it.
        let maybe_listener = unsafe { &*self.listener.as_ptr() };
        if let Some(ref weak_listener) = *maybe_listener {
            println!("listener"); // DEBUG
            if let Some(ref listener) = weak_listener.upgrade() {
                listener.on_evaluated(self, value);
            }
        }
    }

    fn on_operand_evaluated(&self, operand: &PartialExpression<C>, value: C::Value) {
        println!("Operand {} evaluated", operand.index.get()); // DEBUG
        {
            let mut operand_list = self.operands.borrow_mut();

            // Update the operand list.
            if let OperandList::Partial(ref mut operands, ref mut num_partial) = *operand_list {
                if let Expression::Partial(_) = operands[operand.index.get()] {
                    operands[operand.index.get()] = Expression::Total(value);
                    *num_partial -= 1;
                }
            }
        }

        self.try_evaluate_self();
    }
}

// Since we can't derive Debug for Cell<Option<Weak<...>>>, we have to implement it manually.
impl<C: EvaluationContext> Debug for PartialExpression<C> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        // TODO: implement properly.
        write!(
            f,
            "PartialExpression({:?}, {:?})",
            self.operation,
            self.operands.borrow()
        )
    }
}

impl<C: EvaluationContext + 'static> EvaluationListener<C> for PartialExpression<C> {
    fn on_evaluated(&self, partial: &PartialExpression<C>, value: C::Value) {
        // Use hairy pointer comparisons to see how the recently evaluated partial is
        // related to self.
        let self_ptr = self as *const PartialExpression<C> as *const ();
        if partial as *const PartialExpression<C> as *const () == self_ptr {
            self.on_self_evaluated(value);
        } else {
            // This should be effectively safe because we don't mutate the Cell
            // while the pointer is still in use.
            let listener_ptr = unsafe {
                (*partial.listener.as_ptr())
                    .as_ref()
                    .and_then({ |p| p.upgrade() })
                    .map({ |p| Rc::into_raw(p) as *const () })
                    .unwrap_or(ptr::null())
            };
            if listener_ptr == self_ptr {
                self.on_operand_evaluated(partial, value);
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
                        OperandList::Total(operand_values.clone()),
                    );
                    // TODO: reorganize somehow so we don't need the clone above.
                    let listener = Rc::downgrade(&partial);
                    op.register(context, &listener, &operand_values);
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

/**
 * A function that uses an EvaluationContext, an array of operands, and an
 * evaluation listener to handle the evaluation of an Operation
 */
pub type Evaluator<C> =
    fn(&C, &[<C as EvaluationContext>::Value]) -> EvaluationResult<<C as EvaluationContext>::Value>;

pub type Registrar<C> = fn(&C, &Weak<PartialExpression<C>>, &[<C as EvaluationContext>::Value]);

// TODO: handle "lazy"/quoted ops?
#[derive(Clone)]
pub struct Operation<C: EvaluationContext> {
    name: &'static str,
    evaluator: Evaluator<C>,
    registrar: Registrar<C>,
}

impl<C: EvaluationContext> Operation<C> {
    pub const fn new(
        name: &'static str,
        evaluator: Evaluator<C>,
        registrar: Registrar<C>,
    ) -> Operation<C> {
        Operation {
            name,
            evaluator,
            registrar,
        }
    }

    pub fn evaluate(&self, context: &C, operands: &[C::Value]) -> EvaluationResult<C::Value> {
        (self.evaluator)(context, operands)
    }

    pub fn register(
        &self,
        context: &C,
        listener: &Weak<PartialExpression<C>>,
        operands: &[C::Value],
    ) {
        (self.registrar)(context, listener, operands)
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
        OperationGroup { map }
    }

    pub fn get(&self, name: &str) -> Option<&Operation<C>> {
        self.map.get(name)
    }
}
