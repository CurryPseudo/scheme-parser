use chumsky::prelude::*;

use crate::*;

#[derive(Debug)]
pub struct Program {
    pub exprs: Vec<Spanned<Expression>>,
    pub last_expr: Spanned<Expression>,
}

const EXTENDED_IDENTIFIER_CHARS: &str = "!$%&*+-/:<=>?@^_~";

#[derive(Debug)]
pub enum Expression {
    List(Vec<Box<Spanned<Expression>>>),
    Integer(i64),
    Ident(String),
    Error,
}

fn parser() -> impl Parser<char, Program, Error = Simple<char>> {
    let comment = just(";")
        .then(take_until(text::newline().or(end())))
        .padded();
    let expr = recursive(|expr| {
        let num = just('-')
            .or_not()
            .chain::<char, _, _>(text::int(10))
            .collect::<String>()
            .map(|n| Expression::Integer(n.parse::<i64>().unwrap()))
            .padded()
            .labelled("number");
        let ident =
            filter(|c: &char| c.is_ascii_alphanumeric() || EXTENDED_IDENTIFIER_CHARS.contains(*c))
                .repeated()
                .at_least(1)
                .collect()
                .padded()
                .map(Expression::Ident)
                .labelled("ident");
        let list = expr
            .map(Box::new)
            .repeated()
            .delimited_by(just('('), just(')'))
            .collect::<Vec<_>>()
            .map(Expression::List)
            .recover_with(nested_delimiters('(', ')', [], |_| Expression::Error))
            .labelled("list");
        num.or(ident)
            .or(list)
            .map_with_span(|expr, span| (expr, span))
    })
    .padded_by(comment.repeated())
    .padded()
    .labelled("expression");
    expr.clone()
        .repeated()
        .at_least(1)
        .map(|exprs| {
            let len = exprs.len();
            let mut iter = exprs.into_iter();
            let mut exprs = Vec::new();
            for _ in 0..len - 1 {
                exprs.push(iter.next().unwrap());
            }
            let last_expr = iter.next().unwrap();
            Program { exprs, last_expr }
        })
        .then_ignore(end().recover_with(skip_then_retry_until([])))
}

pub fn parse(source: &str, source_path: &str) -> Result<Program, ParseError> {
    parser().parse(source).map_err(|e| ParseError {
        source: source.to_string(),
        source_path: source_path.to_string(),
        simple: e,
        ..Default::default()
    })
}
pub fn parse_recover(source: &str, source_path: &str) -> (Option<Program>, ParseError) {
    let (program, error) = parser().parse_recovery(source);
    (
        program,
        ParseError {
            source: source.to_string(),
            source_path: source_path.to_string(),
            simple: error,
            ..Default::default()
        },
    )
}

#[cfg(test)]
mod tests {}
