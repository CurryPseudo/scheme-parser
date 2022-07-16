use chumsky::{prelude::*, Stream};

use crate::*;

#[derive(Debug)]
pub struct Program {
    pub exprs: Vec<Spanned<Expression>>,
    pub last_expr: Spanned<Expression>,
}

#[derive(Debug)]
pub enum Expression {
    List(Vec<Spanned<Expression>>),
    Integer(i64),
    Ident(String),
    Error,
}

fn parser() -> impl Parser<Token, Program, Error = Simple<Token>> {
    let expr = recursive(|expr| {
        let primitive = filter_map(|span, token| match token {
            Token::Integer(v) => Ok(Expression::Integer(v)),
            Token::Ident(v) => Ok(Expression::Ident(v)),
            other => Err(Simple::expected_input_found(
                span,
                Some(Some(Token::Keyword("<primitive>"))),
                Some(other),
            )),
        });
        let list = expr
            .repeated()
            .delimited_by(just(Token::Keyword("(")), just(Token::Keyword(")")))
            .collect::<Vec<_>>()
            .map(Expression::List)
            .recover_with(nested_delimiters(
                Token::Keyword("("),
                Token::Keyword(")"),
                [],
                |_| Expression::Error,
            ))
            .labelled("list");
        primitive
            .or(list)
            .map_with_span(|expr, span| (expr, span))
            .labelled("expression")
    });
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
        .labelled("program")
        .then_ignore(end().recover_with(skip_then_retry_until([])))
}

pub fn parse(source: &str, source_path: &str) -> (Option<Program>, Option<ParseError<Token>>) {
    let (tokens, _) = tokenize(source, source_path);
    let tokens = tokens.unwrap();
    let len = source.len();
    let (program, error) =
        parser().parse_recovery(Stream::from_iter(len..len + 1, tokens.into_iter()));
    (
        program,
        if error.is_empty() {
            None
        } else {
            Some(ParseError {
                source: source.to_string(),
                source_path: source_path.to_string(),
                simple: error,
                type_name: "token",
                colorful: false,
                display_every_expected: true,
            })
        },
    )
}

#[cfg(test)]
mod tests {}
