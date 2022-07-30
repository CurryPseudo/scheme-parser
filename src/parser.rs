use derive_more::{Display, From};

use crate::*;

pub(super) mod chumsky {
    use chumsky::{combinator::MapWithSpan, prelude::*};
    use std::ops::Range;

    use crate::ast::*;
    use crate::token::*;
    fn enclosed<T>(
        parser: impl chumsky::Parser<Token, T, Error = Simple<Token>>,
    ) -> impl chumsky::Parser<Token, T, Error = Simple<Token>> {
        parser.delimited_by(just(Token::Keyword("(")), just(Token::Keyword(")")))
    }

    fn spanned<T, I: chumsky::Parser<Token, T, Error = Simple<Token>>>(
        parser: I,
    ) -> MapWithSpan<I, impl Fn(T, Range<usize>) -> (T, Range<usize>) + Clone + Copy, T> {
        parser.map_with_span(|t, span| (t, span))
    }

    pub fn parser() -> impl chumsky::Parser<Token, Program, Error = Simple<Token>> {
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

        let ident = spanned(select! {
            "<identifier>",
            { Token::Primitive(Primitive::Ident(v)) => v }
        });

        let proc_body = recursive(|proc_body| {
            let expr = recursive(|expr| {
                let primitive = select! {
                    "<primitive>",
                    {
                        Token::Primitive(p) => Expression::Primitive(p),
                    }
                };
                let formals = enclosed(ident.repeated()).or(ident.map(|ident| vec![ident]));

                let lambda = enclosed(
                    just(Token::Keyword("lambda"))
                        .ignore_then(formals)
                        .then(proc_body.clone().map(Box::new))
                        .map(|(args, body)| Expression::Procedure { args, body }),
                )
                .labelled("lambda");

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

                let proc_call = enclosed(expr.clone().then(expr.clone().repeated()).map(
                    |(operator, args)| Expression::ProcedureCall {
                        operator: Box::new(operator),
                        args,
                    },
                ))
                .labelled("procedure call");

                let assign = enclosed(just(Token::Keyword("set!")).ignore_then(ident).then(expr))
                    .map(|(ident, expr)| Expression::Assignment(ident, Box::new(expr)));

                map_err_category!(
                    "<expression>",
                    spanned(
                        primitive
                            .or(if_expr)
                            .or(lambda)
                            .or(assign)
                            .or(proc_call)
                            .recover_with(nested_delimiters(
                                Token::Keyword("("),
                                Token::Keyword(")"),
                                [],
                                |_| { Expression::Error },
                            ))
                    )
                    .labelled("expression")
                )
            });
            let def_proc = enclosed(
                just(Token::Keyword("define"))
                    .ignore_then(enclosed(ident.then(ident.repeated())))
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
                spanned(
                    def_proc.or(enclosed(
                        just(Token::Keyword("define"))
                            .ignore_then(ident)
                            .then(expr.clone())
                    )
                    .map(|(ident, expr)| Definition(ident, expr)))
                )
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
}

#[derive(Default)]
pub struct Parser {}

#[derive(From, Display)]
pub enum TokenizeOrParseError<'a> {
    Tokenize(ParseError<'a, char>),
    Parse(ParseError<'a, Token>),
}

impl<'a> TokenizeOrParseError<'a> {
    /// Should display with color or not, default: false
    pub fn with_color(self, colorful: bool) -> Self {
        match self {
            TokenizeOrParseError::Tokenize(e) => {
                TokenizeOrParseError::Tokenize(e.with_color(colorful))
            }
            TokenizeOrParseError::Parse(e) => TokenizeOrParseError::Parse(e.with_color(colorful)),
        }
    }
}

impl Parser {
    pub fn parse_tokens<'a>(
        &mut self, // We might add new transformer in self
        tokens: &[Spanned<Token>],
        source: &'a str,
        source_path: &'a str,
    ) -> Result<Program, TokenizeOrParseError<'a>> {
        use ::chumsky::Parser as _;
        let len = source.len();
        chumsky::parser()
            .parse(::chumsky::Stream::from_iter(
                len..len + 1,
                tokens.iter().cloned(),
            ))
            .map_err(|e| {
                ParseError {
                    source,
                    source_path,
                    simple: e,
                    type_name: "token",
                    colorful: false,
                    display_every_expected: true,
                }
                .into()
            })
    }
    pub fn parse<'a>(
        &mut self, // We might add new transformer in self
        source: &'a str,
        source_path: &'a str,
    ) -> Result<Program, TokenizeOrParseError<'a>> {
        let tokens = tokenize(source, source_path)?;
        self.parse_tokens(&tokens, source, source_path)
    }
}
