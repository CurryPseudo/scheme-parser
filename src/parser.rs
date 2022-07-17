use chumsky::{prelude::*, Stream};

use crate::*;

#[derive(Debug)]
pub struct Program {
    pub defs: Vec<Definition>,
    pub exprs: Vec<Spanned<Expression>>,
    pub last_expr: Spanned<Expression>,
}

#[derive(Debug)]
pub struct Definition(pub String, pub Spanned<Expression>);

#[derive(Debug)]
pub enum Expression {
    List(Vec<Spanned<Expression>>),
    Integer(i64),
    Ident(String),
    Error,
}

fn parser() -> impl Parser<Token, Program, Error = Simple<Token>> {
    let ident = filter_map(|span, token| match token {
        Token::Ident(v) => Ok(v),
        other => Err(Simple::expected_input_found(
            span,
            Some(Some(Token::Keyword("<identifier>"))),
            Some(other),
        )),
    });
    let expr = recursive(|expr| {
        let ident = ident.map(Expression::Ident);
        let integer = filter_map(|span, token| match token {
            Token::Integer(v) => Ok(Expression::Integer(v)),
            other => Err(Simple::expected_input_found(
                span,
                Some(Some(Token::Keyword("<integer>"))),
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
        integer
            .or(ident)
            .or(list)
            .map_with_span(|expr, span| (expr, span))
            .labelled("expression")
            .map_err(|e| {
                if let chumsky::error::SimpleReason::Unexpected = e.reason() {
                    let r = Simple::expected_input_found(
                        e.span(),
                        Some(Some(Token::Keyword("<expression>"))),
                        e.found().cloned(),
                    );
                    if let Some(label) = e.label() {
                        r.with_label(label)
                    } else {
                        r
                    }
                } else {
                    e
                }
            })
    });
    let def = ident
        .then(expr.clone())
        .delimited_by(
            just(vec![Token::Keyword("("), Token::Keyword("define")]),
            just(Token::Keyword(")")),
        )
        .map(|(ident, expr)| Definition(ident, expr))
        .labelled("definition");
    def.repeated()
        .then(expr.repeated().at_least(1))
        .map(|(defs, exprs)| {
            let len = exprs.len();
            let mut iter = exprs.into_iter();
            let mut exprs = Vec::new();
            for _ in 0..len - 1 {
                exprs.push(iter.next().unwrap());
            }
            let last_expr = iter.next().unwrap();
            Program {
                defs,
                exprs,
                last_expr,
            }
        })
        .labelled("program")
        .then_ignore(end().recover_with(skip_then_retry_until([])))
}

pub fn parse(source: &str, source_path: &str) -> (Option<Program>, Option<ParseError<Token>>) {
    let (tokens, _) = tokenize(source, source_path);
    if let Some(tokens) = tokens {
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
    } else {
        (None, None)
    }
}

#[cfg(test)]
mod tests {}
