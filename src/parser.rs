use chumsky::{prelude::*, Stream};
use derive_more::From;

use crate::*;

#[derive(Debug)]
pub struct ProcedureBody {
    pub defs: Vec<Spanned<Definition>>,
    pub exprs: Vec<Spanned<Expression>>,
    pub last_expr: Spanned<Expression>,
}

pub type Program = ProcedureBody;

#[derive(Debug)]
pub struct Definition(pub Spanned<String>, pub Spanned<Expression>);

#[derive(Debug, From)]
pub enum Expression {
    ProcedureCall {
        operator: Box<Spanned<Expression>>,
        args: Vec<Spanned<Expression>>,
    },
    Primitive(Primitive),
    Procedure {
        /// Arguments
        args: Vec<Spanned<String>>,
        /// Procedure Body
        body: Box<ProcedureBody>,
    },
    Conditional {
        /// Test
        test: Box<Spanned<Expression>>,
        /// Consequent
        conseq: Box<Spanned<Expression>>,
        /// Alternative
        alter: Option<Box<Spanned<Expression>>>,
    },
    Error,
}

fn enclosed<T>(
    parser: impl Parser<Token, T, Error = Simple<Token>>,
) -> impl Parser<Token, T, Error = Simple<Token>> {
    parser.delimited_by(just(Token::Keyword("(")), just(Token::Keyword(")")))
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

    let spanned_ident = ident.map_with_span(|ident, span| (ident, span));

    let proc_body = recursive(|proc_body| {
        let expr = recursive(|expr| {
            let primitive = select! {
                "<primitive>",
                {
                    Token::Primitive(p) => Expression::Primitive(p),
                }
            };
            let formals =
                enclosed(ident.map_with_span(|ident, span| (ident, span)).repeated()).or(ident
                    .map_with_span(|ident, span| (ident, span))
                    .map(|ident| vec![ident]));

            let lambda = enclosed(
                just(Token::Keyword("lambda"))
                    .ignore_then(formals)
                    .then(proc_body.clone().map(Box::new))
                    .map(|(args, body)| Expression::Procedure { args, body }),
            );

            let if_expr = enclosed(
                just(Token::Keyword("if"))
                    .ignore_then(expr.clone())
                    .then(expr.clone())
                    .then(expr.clone().or_not()),
            )
            .map(|((test, conseq), alter)| Expression::Conditional {
                test: Box::new(test),
                conseq: Box::new(conseq),
                alter: alter.map(Box::new),
            });

            let proc_call = enclosed(expr.clone().then(expr.repeated()).map(|(operator, args)| {
                Expression::ProcedureCall {
                    operator: Box::new(operator),
                    args,
                }
            }))
            .labelled("procedure call");
            map_err_category!(
                "<expression>",
                primitive
                    .or(if_expr)
                    .or(lambda)
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
        let def_proc = enclosed(
            just(Token::Keyword("define"))
                .ignore_then(enclosed(spanned_ident.then(spanned_ident.repeated())))
                .then(proc_body),
        )
        .map_with_span(|((ident, args), body), span| {
            Definition(
                ident,
                (
                    Expression::Procedure {
                        args,
                        body: Box::new(body),
                    },
                    span,
                ),
            )
        });
        let def = map_err_category!(
            "<definition>",
            def_proc
                .or(enclosed(
                    just(Token::Keyword("define"))
                        .ignore_then(spanned_ident)
                        .then(expr.clone())
                )
                .map(|(ident, expr)| Definition(ident, expr)))
                .map_with_span(|def, span| (def, span))
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
            .labelled("procedure body")
    });
    proc_body.labelled("program").then_ignore(end())
}

pub fn parse<'a>(
    source: &'a str,
    source_path: &'a str,
) -> (Option<Program>, Option<ParseError<'a, Token>>) {
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
                    source,
                    source_path,
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
