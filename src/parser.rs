use chumsky::{prelude::*, Stream};
use derive_more::From;

use crate::*;

#[derive(Debug)]
pub struct ProcedureBody {
    pub defs: Vec<Definition>,
    pub exprs: Vec<Spanned<Expression>>,
    pub last_expr: Spanned<Expression>,
}

pub type Program = ProcedureBody;

#[derive(Debug)]
pub struct Definition(pub String, pub Spanned<Expression>);

#[derive(Debug, From)]
pub enum Expression {
    ProcedureCall(Vec<Spanned<Expression>>),
    Primitive(Primitive),
    Procedure {
        args: Vec<Spanned<Expression>>,
        body: Box<ProcedureBody>,
    },
    Error,
}

fn parser() -> impl Parser<Token, Program, Error = Simple<Token>> {
    macro_rules! select {
        ($category: literal, {$($p:pat $(if $guard:expr)? => $out:expr),+ $(,)?}) => ({
            filter_map(move |span: std::ops::Range<usize>, x| match x {
                $($p $(if $guard)? => ::core::result::Result::Ok($out)),+,
                other => Err(Simple::expected_input_found(
                    span,
                    Some(Some(Token::Keyword($category))),
                    Some(other),
                )),
            })
        });
    }
    macro_rules! map_err_category {
        ($category: literal, $parser: expr) => {
            $parser.map_err(|e| {
                if let chumsky::error::SimpleReason::Unexpected = e.reason() {
                    let r = Simple::expected_input_found(
                        e.span(),
                        Some(Some(Token::Keyword($category))),
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
        };
    }

    let ident = select! {
        "<identifier>",
        { Token::Primitive(Primitive::Ident(v)) => v }
    };

    let expr = recursive(|expr| {
        let primitive = select! {
            "<primitive>",
            {
                Token::Primitive(p) => Expression::Primitive(p),
            }
        };
        let proc_call = expr
            .repeated()
            .delimited_by(just(Token::Keyword("(")), just(Token::Keyword(")")))
            .collect::<Vec<_>>()
            .map(Expression::ProcedureCall)
            .labelled("procedure call");
        map_err_category!(
            "<expression>",
            primitive
                .or(proc_call)
                .recover_with(nested_delimiters(
                    Token::Keyword("("),
                    Token::Keyword(")"),
                    [],
                    |_| Expression::Error,
                ))
                .map_with_span(|expr, span| (expr, span))
                .labelled("expression")
        )
    });
    let def = map_err_category!(
        "<definition>",
        ident
            .then(expr.clone())
            .delimited_by(
                just(vec![Token::Keyword("("), Token::Keyword("define")]),
                just(Token::Keyword(")")),
            )
            .map(|(ident, expr)| Definition(ident, expr))
            .labelled("definition")
    );
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
        .then_ignore(end())
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
