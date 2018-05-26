use base::context::EvaluationContext;
use base::value::{Expression, OperationGroup, Value};
use ir;
use ir::parser::{Element, ParseError, ParseErrorCause};

pub fn expression_from_parser(
    parser: &mut ir::Parser,
    operations: &OperationGroup,
    context: &EvaluationContext,
) -> Option<Result<Expression, ParseError>> {
    match parser.next_element() {
        Some(result) => Some(match result {
            Ok(element) => expression_from_element(element, operations, context),
            Err(error) => Err(error),
        }),
        None => None,
    }
}

pub fn expression_from_element<'a>(
    element: Element<'a>,
    operations: &OperationGroup,
    context: &EvaluationContext,
) -> Result<Expression, ParseError> {
    match element.data {
        ir::ElementData::Operation(mut op_iter) => {
            let op = match operations.get(op_iter.op_text.text()) {
                Some(op) => *op,
                None => {
                    return Err(ParseError::new(
                        op_iter.op_text,
                        ParseErrorCause::UndefinedOperation,
                    ));
                }
            };
            let mut operands: Vec<Expression> = vec![];

            while let Some(op_or_err) = op_iter.next_element() {
                match op_or_err {
                    Ok(op_el) => match expression_from_element(op_el, operations, context) {
                        Ok(exp) => operands.push(exp),
                        Err(err) => return Err(err),
                    },
                    Err(err) => return Err(err),
                }
            }

            Ok(Expression::from_op(op, context, operands))
        }
        ir::ElementData::Integer => {
            // TODO: handle integer parsing better.
            let value = str::parse::<usize>(element.location.text()).unwrap();
            Ok(Expression::from_value(Ok(Value::Integer(value))))
        }
        ir::ElementData::String => {
            let text = element.location.text();
            let value = text[1..text.len() - 1].to_owned();
            Ok(Expression::from_value(Ok(Value::String(value))))
        }
    }
}
