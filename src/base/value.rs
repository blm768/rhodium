use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::rc::Weak;

use base::context::{EvaluationContext, LookupResult};
use base::expression;
use base::expression::EvaluationResult;
use base::expression::EvaluationResult::{Pending, Total};

// A temporary value type (will later be replaced with something more generic)
#[derive(Clone, Debug)]
pub enum Value {
    Integer(usize),
    String(String),
}

/**
 * Represents either a value or an evaluation error
 */
pub type ValueResult = Result<Value, ValueError>;

impl expression::Value for ValueResult {}

/**
 * An error triggered during evaluation
 */
#[derive(Copy, Clone, Debug)]
pub struct ValueError {
    // TODO: include location and/or operator context.
    //location: SourceLocation
    cause: ValueErrorCause,
}

impl ValueError {
    pub fn new(cause: ValueErrorCause) -> ValueError {
        ValueError { cause }
    }
}

/**
 * The cause of a `ValueError`
 */
#[derive(Copy, Clone, Debug)]
pub enum ValueErrorCause {
    UnspecifiedError,
    WrongNumberOfOperandsForOperation { expected: usize, found: usize },
    WrongTypesForOperation,
}

impl Display for ValueError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        write!(formatter, "Error: {}", self.description())
    }
}

impl Error for ValueError {
    fn description(&self) -> &str {
        "Value error"
    }
}

pub type Operation = expression::Operation<EvaluationContext>;
pub type EvaluationListener = expression::EvaluationListener<EvaluationContext>;
pub type OperationGroup = expression::OperationGroup<EvaluationContext>;
pub type Expression = expression::Expression<EvaluationContext>;
pub type PartialExpression = expression::PartialExpression<EvaluationContext>;

fn propagate_errors(
    op: fn(&EvaluationContext, &[Value]) -> EvaluationResult<ValueResult>,
    context: &EvaluationContext,
    operands: &[ValueResult],
) -> EvaluationResult<ValueResult> {
    let mut values = Vec::<Value>::with_capacity(operands.len());
    for operand in operands.iter() {
        match *operand {
            Ok(ref val) => values.push(val.clone()),
            // TODO: combine multiple errors if present.
            Err(err) => return Total(Err(err)),
        }
    }

    op(context, values.as_ref())
}

fn unary_op(
    op: fn(&EvaluationContext, &Value) -> EvaluationResult<ValueResult>,
    context: &EvaluationContext,
    operands: &[Value],
) -> EvaluationResult<ValueResult> {
    if operands.len() == 1 {
        op(context, &operands[0])
    } else {
        Total(Err(ValueError::new(
            ValueErrorCause::WrongNumberOfOperandsForOperation {
                expected: 1,
                found: operands.len(),
            },
        )))
    }
}

fn binary_op(
    op: fn(&EvaluationContext, &Value, &Value) -> EvaluationResult<ValueResult>,
    context: &EvaluationContext,
    operands: &[Value],
) -> EvaluationResult<ValueResult> {
    if operands.len() == 2 {
        op(context, &operands[0], &operands[1])
    } else {
        Total(Err(ValueError::new(
            ValueErrorCause::WrongNumberOfOperandsForOperation {
                expected: 2,
                found: operands.len(),
            },
        )))
    }
}

fn null_registrar(_: &EvaluationContext, _: &Weak<PartialExpression>, _: &[ValueResult]) {}

fn add(context: &EvaluationContext, args: &[ValueResult]) -> EvaluationResult<ValueResult> {
    fn do_add(_: &EvaluationContext, a: &Value, b: &Value) -> EvaluationResult<ValueResult> {
        Total(match *a {
            Value::Integer(a_num) => match *b {
                Value::Integer(b_num) => Ok(Value::Integer(a_num + b_num)),
                _ => Err(ValueError::new(ValueErrorCause::WrongTypesForOperation)),
            },
            _ => Err(ValueError::new(ValueErrorCause::WrongTypesForOperation)),
        })
    }

    fn bin_add(context: &EvaluationContext, operands: &[Value]) -> EvaluationResult<ValueResult> {
        binary_op(do_add, context, operands)
    }

    propagate_errors(bin_add, context, args)
}

fn define_symbol(
    context: &EvaluationContext,
    args: &[ValueResult],
) -> EvaluationResult<ValueResult> {
    fn do_define(
        context: &EvaluationContext,
        name: &Value,
        value: &Value,
    ) -> EvaluationResult<ValueResult> {
        if let Value::String(ref name_str) = *name {
            let scope = context.scope();
            let mut scope_mut = scope.borrow_mut();
            match scope_mut.define_symbol(name_str, value.clone()) {
                Ok(_) => Total(Ok(value.clone())),
                // TODO: do something useful with the error value.
                Err(_) => Total(Err(ValueError::new(ValueErrorCause::UnspecifiedError))),
            }
        } else {
            Total(Err(ValueError::new(
                ValueErrorCause::WrongTypesForOperation,
            )))
        }
    }

    fn bin_define(
        context: &EvaluationContext,
        operands: &[Value],
    ) -> EvaluationResult<ValueResult> {
        binary_op(do_define, context, operands)
    }

    propagate_errors(bin_define, context, args)
}

fn get_symbol(context: &EvaluationContext, args: &[ValueResult]) -> EvaluationResult<ValueResult> {
    fn do_get(context: &EvaluationContext, name: &Value) -> EvaluationResult<ValueResult> {
        if let Value::String(ref name_str) = *name {
            let scope = context.scope();
            let mut scope_mut = scope.borrow_mut();
            // TODO: handle registration.
            match scope_mut.get_symbol(name_str) {
                LookupResult::Total(value) => Total(Ok(value.clone())),
                LookupResult::Pending => Pending,
                // TODO: do something useful with the error value.
                LookupResult::NotFound => {
                    Total(Err(ValueError::new(ValueErrorCause::UnspecifiedError)))
                }
            }
        } else {
            Total(Err(ValueError::new(
                ValueErrorCause::WrongTypesForOperation,
            )))
        }
    }

    fn unary_get(context: &EvaluationContext, operands: &[Value]) -> EvaluationResult<ValueResult> {
        unary_op(do_get, context, operands)
    }

    propagate_errors(unary_get, context, args)
}

fn get_symbol_register(
    context: &EvaluationContext,
    listener: &Weak<PartialExpression>,
    operands: &[ValueResult],
) {
    if operands.len() != 1 {
        return;
    }

    if let Ok(Value::String(ref name)) = operands[0] {
        let scope = context.scope();
        scope.borrow_mut().register_listener(name, listener.clone());
    }
}

const ADD_OP: Operation = Operation::new("add", add, null_registrar);
const DEFINE_OP: Operation = Operation::new("define_symbol", define_symbol, null_registrar);
const GET_SYM_OP: Operation = Operation::new("get_symbol", get_symbol, get_symbol_register);

/**
 * Returns the default Rhodium `OperationGroup`
 */
pub fn default_operations() -> OperationGroup {
    OperationGroup::new(
        [
            ("add", ADD_OP),
            ("define_symbol", DEFINE_OP),
            ("get_symbol", GET_SYM_OP),
        ].iter()
            .cloned()
            .map(|i| (Box::<str>::from(i.0), i.1))
            .collect(),
    )
}
