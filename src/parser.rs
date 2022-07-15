use chumsky::prelude::*;

#[derive(Debug)]
pub struct Program {
    pub exprs: Vec<Expression>,
    pub last_expr: Expression,
}

const EXTENDED_IDENTIFIER_CHARS: &str = "!$%&*+-/:<=>?@^_~";

#[derive(Debug)]
pub enum Expression {
    List(Vec<Box<Expression>>),
    Integer(i64),
    Ident(String),
}

pub fn parser() -> impl Parser<char, Program, Error = Simple<char>> {
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
            .labelled("list");
        num.or(ident).or(list)
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

#[cfg(test)]
mod tests {
    use super::*;
    use manifest_dir_macros::*;
    use pretty_assertions::assert_eq;
    use std::ffi::OsStr;
    use std::fs::{read_dir, read_to_string, File};
    use std::io::Write;
    use std::path::Path;
    fn assert_eq_or_override(path: &Path, content: &str) {
        if cfg!(feature = "override-test") {
            let mut file = File::create(path).unwrap();
            file.write_all(content.as_bytes()).unwrap();
        }
        let actual = read_to_string(path)
            .unwrap_or_else(|_| String::new())
            .replace('\r', "");
        assert_eq!(content, &actual)
    }
    #[test]
    fn parser_works() {
        let input = read_to_string(path!("tests/parser.scm")).unwrap();
        let result = parser().parse(input).unwrap_or_else(|e| {
            use std::fmt::Write;
            let mut s = String::new();
            for e in e {
                writeln!(s, "{}", e).unwrap();
            }
            panic!("{}", s)
        });
        let content = format!("{:#?}", result);
        assert_eq_or_override(Path::new(path!("tests/parser.ast")), &content);
    }
    #[test]
    fn errors_works() {
        let dir = path!("tests/error");
        for entry in read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension() != Some(OsStr::new("scm")) {
                continue;
            }
            let input = read_to_string(&path).unwrap();
            let result = parser().parse(input).unwrap_err();
            let mut content = String::new();
            for error in result {
                use std::fmt::Write;
                writeln!(content, "{}", error).unwrap();
            }
            let mut error_path = path;
            error_path.set_extension("err");
            assert_eq_or_override(&error_path, &content);
        }
    }
}
