use chumsky::{prelude::*, Stream};

use crate::*;

#[derive(Debug)]
pub struct Program {
    pub exprs: Vec<Spanned<Expression>>,
    pub last_expr: Spanned<Expression>,
}

#[derive(Debug)]
pub enum Expression {
    List(Vec<Box<Spanned<Expression>>>),
    Integer(i64),
    Ident(String),
    Error,
}

fn parser() -> impl Parser<Token, Program, Error = Simple<Token>> {
    let expr = recursive(|expr| {
        let primitive = select! {
            Token::Integer(v) => Expression::Integer(v),
            Token::Ident(v) => Expression::Ident(v),
        }
        .labelled("primitive");
        let list = expr
            .map(Box::new)
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
        primitive.or(list).map_with_span(|expr, span| (expr, span))
    });
    expr.clone().repeated().at_least(1).map(|exprs| {
        let len = exprs.len();
        let mut iter = exprs.into_iter();
        let mut exprs = Vec::new();
        for _ in 0..len - 1 {
            exprs.push(iter.next().unwrap());
        }
        let last_expr = iter.next().unwrap();
        Program { exprs, last_expr }
    })
}

pub fn parse(source: &str, source_path: &str) -> Result<Program, ParseError> {
    let (tokens, _) = tokenize(source, source_path);
    let tokens = tokens.unwrap();
    let len = source.len();
    parser()
        .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
        .map_err(|e| ParseError {
            source: source.to_string(),
            source_path: source_path.to_string(),
            simple: e,
            ..Default::default()
        })
}
pub fn parse_recover(source: &str, source_path: &str) -> (Option<Program>, Option<ParseError>) {
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
                ..Default::default()
            })
        },
    )
}

#[cfg(test)]
mod tests {}
