use chumsky::prelude::*;

#[derive(Debug)]
pub struct Program {
    exprs: Vec<Expression>,
}
#[derive(Debug)]
pub enum Expression {
    List(Vec<Box<Expression>>),
    Integer(i64),
    Symbol(String),
}

fn parser() -> impl Parser<char, Program, Error = Simple<char>> {
    let expr = recursive(|expr| {
        let num = just('-')
            .or_not()
            .chain::<char, _, _>(text::int(10))
            .collect::<String>()
            .map(|n| Expression::Integer(n.parse::<i64>().unwrap()))
            .padded()
            .labelled("number");
        let ident = text::ident::<char, Simple<char>>()
            .padded()
            .map(Expression::Symbol)
            .labelled("ident");
        let list = expr
            .map(Box::new)
            .repeated()
            .delimited_by(just('('), just(')'))
            .collect::<Vec<_>>()
            .map(Expression::List)
            .labelled("list");
        num.or(ident).or(list)
    })
    .labelled("expression");
    expr.padded()
        .repeated()
        .map(|exprs| Program { exprs })
        .then_ignore(end().recover_with(skip_then_retry_until([])))
}

#[test]
fn test_parser() {
    use pretty_assertions::assert_eq;
    let input = include_str!("test.scm");
    let result = parser().parse(input).unwrap_or_else(|e| {
        use std::fmt::Write;
        let mut s = String::new();
        for e in e {
            writeln!(s, "{}", e).unwrap();
        }
        panic!("{}", s)
    });
    assert_eq!(&format!("{:#?}", result), include_str!("test.ast"))
}
