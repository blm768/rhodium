use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

use base::SourceLocation;
use base::operation;
use base::operation::EvaluationResult;
use base::operation::EvaluationResult::{Complete, Pending};
use base::operation::OperationGroup;

// A temporary value type (will later be replaced with something more generic)
#[derive(Clone, Debug)]
pub enum Value {
    Integer(usize),
}

// TODO: include a "back trace"?
// TODO: include (or make into) an enum?
#[derive(Copy, Clone, Debug)]
pub struct ValueError {
    //location: SourceLocation
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

type ValueResult = Result<Value, ValueError>;

impl operation::Value for ValueResult {}

pub type Operation = operation::Operation<ValueResult>;
pub type Expression = operation::Expression<ValueResult>;

fn propagate_errors(op: fn(&[Value]) -> EvaluationResult<ValueResult>, operands: &[ValueResult]) -> EvaluationResult<ValueResult> {
    let mut values = Vec::<Value>::with_capacity(operands.len());
    for operand in operands.iter() {
        match *operand {
            Ok(ref val) => values.push(val.clone()),
            // TODO: combine multiple errors if present.
            Err(ref err) => return Complete(Err(err.clone())),
        }
    }

    op(values.as_ref())
}

// TODO: put a generic version of this in the IR code?
// TODO: only take Values, not ValueResults?
// (Have a wrapper that propagates all errors?)
// NOTE: the op function really should be returning an EvaluationResult, not a ValueResult.
fn binary_op(op: fn(&Value, &Value) -> EvaluationResult<ValueResult>, operands: &[Value]) -> EvaluationResult<ValueResult> {
    match operands.len() {
        2 => op(&operands[0], &operands[1]),
        _ => Complete(Err(ValueError {}))
    }
}

fn add(args: &[ValueResult]) -> EvaluationResult<ValueResult> {
    fn do_add(a: &Value, b: &Value) -> EvaluationResult<ValueResult> {
        match a {
            &Value::Integer(a_num) => {
                match b {
                    &Value::Integer(b_num) => {
                        Complete(Ok(Value::Integer(a_num + b_num)))
                    }
                }
            }
        }
    }

    fn bin_add(operands: &[Value]) -> EvaluationResult<ValueResult> {
        binary_op(do_add, operands)
    }

    propagate_errors(bin_add, args)
}

const ADD_OP: Operation = Operation::new("add", add);

pub static OPERATIONS: OperationGroup<ValueResult> = OperationGroup::<ValueResult>::new(phf_map! {
    "add" => ADD_OP,
});
