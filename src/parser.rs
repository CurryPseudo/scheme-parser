use chumsky::prelude::*;

#[derive(Debug)]
pub struct Program {
    exprs: Vec<Expression>,
}

const EXTENDED_IDENTIFIER_CHARS: &str = "!$%&*+-/:<=>?@^_~";

#[derive(Debug)]
pub enum Expression {
    List(Vec<Box<Expression>>),
    Integer(i64),
    Ident(String),
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
        let ident =
            filter(|c: &char| c.is_ascii_alphabetic() || EXTENDED_IDENTIFIER_CHARS.contains(*c))
                .map(Some)
                .chain::<char, _, _>(
                    filter(|c: &char| {
                        c.is_ascii_alphanumeric() || EXTENDED_IDENTIFIER_CHARS.contains(*c)
                    })
                    .repeated(),
                )
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
            .labelled("list");
        num.or(ident).or(list)
    })
    .labelled("expression");
    expr.padded()
        .repeated()
        .map(|exprs| Program { exprs })
        .then_ignore(end().recover_with(skip_then_retry_until([])))
}

#[cfg(test)]
mod tests {
    use super::*;
    use manifest_dir_macros::*;
    use pretty_assertions::assert_eq;
    use std::fs::File;
    use std::io::Write;
    #[test]
    fn test_parser() {
        let input = include_str!("test.scm");
        let result = parser().parse(input).unwrap_or_else(|e| {
            use std::fmt::Write;
            let mut s = String::new();
            for e in e {
                writeln!(s, "{}", e).unwrap();
            }
            panic!("{}", s)
        });
        let content = format!("{:#?}", result);
        if cfg!(feature = "override_test") {
            let mut file = File::create(path!("src/test.ast")).unwrap();
            file.write_all(content.as_bytes()).unwrap();
        }
        let actual = std::fs::read_to_string(path!("src/test.ast")).unwrap();
        assert_eq!(&content, &actual)
    }
}
