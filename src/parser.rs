use chumsky::{prelude::*, recovery::nested_delimiters};

#[derive(Debug)]
pub struct Program {
    pub exprs: Vec<Spanned<Expression>>,
    pub last_expr: Spanned<Expression>,
}

const EXTENDED_IDENTIFIER_CHARS: &str = "!$%&*+-/:<=>?@^_~";

pub type Span = std::ops::Range<usize>;
pub type Spanned<T> = (T, Span);

#[derive(Debug)]
pub enum Expression {
    List(Vec<Box<Spanned<Expression>>>),
    Integer(i64),
    Ident(String),
    Error,
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

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne::{sources, Color, Fmt, Label, Report, ReportKind};
    use manifest_dir_macros::*;
    use pretty_assertions::assert_eq;
    use std::ffi::OsStr;
    use std::fs::{read_dir, read_to_string, File};
    use std::io::Write;
    use std::path::Path;
    use strip_ansi_escapes::strip;
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
            let path_str = path
                .strip_prefix(path!("."))
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned();
            let input = read_to_string(&path).unwrap();
            let result = parser().parse(input.as_str()).unwrap_err();
            let mut content = Vec::new();
            for error in result {
                let report = Report::build(ReportKind::Error, path_str.clone(), error.span().start);
                let report = match error.reason() {
                    chumsky::error::SimpleReason::Unexpected => report
                        .with_message(format!(
                            "{}, expected {}",
                            if error.found().is_some() {
                                "Unexpected char in input"
                            } else {
                                "Unexpected end of input"
                            },
                            if error.expected().len() == 0 {
                                "something else".to_string()
                            } else {
                                let mut expected = error
                                    .expected()
                                    .map(|expected| match expected {
                                        Some(expected) => expected.to_string(),
                                        None => "end of input".to_string(),
                                    })
                                    .collect::<Vec<_>>();
                                expected.sort();
                                expected.join(", ")
                            },
                        ))
                        .with_label(
                            Label::new((path_str.clone(), error.span()))
                                .with_message(format!(
                                    "Unexpected {}",
                                    error
                                        .found()
                                        .map(|c| format!("token {}", c))
                                        .unwrap_or_else(|| "end of file".to_string())
                                        .fg(Color::Red)
                                ))
                                .with_color(Color::Red),
                        ),
                    chumsky::error::SimpleReason::Unclosed { span, delimiter } => report
                        .with_message(format!(
                            "Unclosed delimiter {}",
                            delimiter.fg(Color::Yellow),
                        ))
                        .with_label(
                            Label::new((path_str.clone(), span.clone()))
                                .with_message(format!(
                                    "Unclosed delimiter {}",
                                    delimiter.fg(Color::Yellow)
                                ))
                                .with_color(Color::Yellow),
                        )
                        .with_label(
                            Label::new((path_str.clone(), error.span()))
                                .with_message(format!(
                                    "Must be closed before this {}",
                                    error
                                        .found()
                                        .map(|c| c.to_string())
                                        .unwrap_or_else(|| "end of file".to_string())
                                        .fg(Color::Red)
                                ))
                                .with_color(Color::Red),
                        ),
                    chumsky::error::SimpleReason::Custom(msg) => {
                        report.with_message(msg).with_label(
                            Label::new((path_str.clone(), error.span()))
                                .with_message(format!("{}", msg.fg(Color::Red)))
                                .with_color(Color::Red),
                        )
                    }
                };
                report
                    .finish()
                    .write(sources(vec![(path_str.clone(), &input)]), &mut content)
                    .unwrap();
            }
            // strip out colors
            let content = strip(content).unwrap();
            let content = String::from_utf8(content).unwrap();
            let mut error_path = path.clone();
            error_path.set_extension("err");
            assert_eq_or_override(&error_path, &content);
        }
    }
}
