use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

use base::SourceLocation;
use base::operation;
use base::operation::EvaluationResult;
use base::operation::EvaluationResult::{Complete, Incomplete};
use base::operation::Evaluator;
use base::operation::OperationGroup;

// A temporary value type (will later be replaced with something more generic)
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
        // TODO: implement.
        //formatter.print(self.description());
        Ok(())
    }
}

impl Error for ValueError {
    fn description(&self) -> &str {
        "Value error"
    }
}

type ValueResult = Result<Value, ValueError>;

type Operation = operation::Operation<ValueResult>;

// TODO: put a generic version of this in the IR code?
// NOTE: the op function really should be returning an EvaluationResult, not a ValueResult.
fn binary_op(op: fn(&ValueResult, &ValueResult) -> ValueResult, args: &mut Iterator<Item = ValueResult>) -> EvaluationResult<ValueResult> {
    match args.next() {
        Some(a) => {
            match args.next() {
                Some(b) => {
                    match args.next() {
                        Some(_) => Complete(Err(ValueError {})),
                        None => Complete(op(&a, &b))
                    }
                },
                None => Complete(Err(ValueError {}))
            }
        },
        None => Complete(Err(ValueError {}))
    }
}

// TODO: create a generic wrapper function (or macro) that produces an error if given any errors
// as input and otherwise forwards to the given function
fn try_op(op: fn(&Value, &Value) -> ValueResult, a: &ValueResult, b: &ValueResult) -> ValueResult {
    match a {
        &Ok(ref a_val) => {
            match b {
                &Ok(ref b_val) => op(&a_val, &b_val),
                &Err(e) => Err(e)
            }
        },
        &Err(e) => Err(e)
    }
}

fn add(args: &mut Iterator<Item = ValueResult>) -> EvaluationResult<ValueResult> {
    fn do_add(a: &Value, b: &Value) -> ValueResult {
        match a {
            &Value::Integer(a_num) => {
                match b {
                    &Value::Integer(b_num) => {
                        Ok(Value::Integer(a_num + b_num))
                    }
                }
            }
        }
    }

    fn try_add(a: &ValueResult, b: &ValueResult) -> ValueResult {
        try_op(do_add, a, b)
    }

    binary_op(try_add, args)
}

fn incomplete_add(args: &mut Iterator<Item = ValueResult>) -> EvaluationResult<ValueResult> {
    Incomplete
}

const ADD_OP: Operation = Operation::new("add", incomplete_add);//as Evaluator<ValueResult> };

pub fn operations() -> OperationGroup<ValueResult> {
    let mut group = OperationGroup::new();
    group.insert(ADD_OP);
    group
}
