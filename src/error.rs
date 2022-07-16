use std::fmt::Debug;

use ariadne::{sources, Color, Fmt, Label, Report, ReportKind};
use chumsky::prelude::Simple;

use crate::Token;

#[derive(Debug, Default)]
pub struct ParseError {
    pub(crate) simple: Vec<Simple<Token>>,
    pub(crate) source: String,
    pub(crate) source_path: String,
    pub(crate) colorful: bool,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        macro_rules! with_color {
            ($a: expr, $b: expr) => {
                if self.colorful {
                    $a.with_color($b)
                } else {
                    $a
                }
            };
        }
        macro_rules! fg {
            ($a: expr, $b: expr) => {
                if self.colorful {
                    Box::new($a.fg($b)) as Box<dyn std::fmt::Display>
                } else {
                    Box::new($a) as Box<dyn std::fmt::Display>
                }
            };
        }
        for error in &self.simple {
            let report = Report::build(
                ReportKind::Error,
                self.source_path.clone(),
                error.span().start,
            );
            let report = match error.reason() {
                chumsky::error::SimpleReason::Unexpected => report
                    .with_message(format!(
                        "{}, expected {}{}",
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
                        if let Some(label) = error.label() {
                            format!(", parsing {}", label)
                        } else {
                            "".to_string()
                        }
                    ))
                    .with_label(with_color!(
                        Label::new((self.source_path.clone(), error.span())).with_message(format!(
                            "Unexpected {}",
                            fg!(
                                error
                                    .found()
                                    .map(|c| format!("char {}", c))
                                    .unwrap_or_else(|| "end of file".to_string()),
                                Color::Red
                            )
                        )),
                        Color::Red
                    )),
                chumsky::error::SimpleReason::Unclosed { span, delimiter } => report
                    .with_message(format!(
                        "Unclosed delimiter {}{}",
                        fg!(delimiter, Color::Yellow),
                        if let Some(label) = error.label() {
                            format!(", parsing {}", label)
                        } else {
                            "".to_string()
                        }
                    ))
                    .with_label(with_color!(
                        Label::new((self.source_path.clone(), span.clone())).with_message(format!(
                            "Unclosed delimiter {}",
                            fg!(delimiter, Color::Yellow)
                        )),
                        Color::Yellow
                    ))
                    .with_label(with_color!(
                        Label::new((self.source_path.clone(), error.span())).with_message(format!(
                            "Must be closed before this {}",
                            fg!(
                                error
                                    .found()
                                    .map(|c| c.to_string())
                                    .unwrap_or_else(|| "end of file".to_string()),
                                Color::Red
                            )
                        )),
                        Color::Red
                    )),
                chumsky::error::SimpleReason::Custom(msg) => report
                    .with_message(format!(
                        "{}{}",
                        msg,
                        if let Some(label) = error.label() {
                            format!(", parsing {}", label)
                        } else {
                            "".to_string()
                        }
                    ))
                    .with_label(with_color!(
                        Label::new((self.source_path.clone(), error.span()))
                            .with_message(format!("{}", fg!(msg, Color::Red))),
                        Color::Red
                    )),
            };
            let mut content = Vec::new();
            if report
                .with_config(ariadne::Config::default().with_color(self.colorful))
                .finish()
                .write(
                    sources(vec![(self.source_path.clone(), &self.source)]),
                    &mut content,
                )
                .is_err()
            {
                return Err(std::fmt::Error);
            }
            write!(f, "{}", String::from_utf8_lossy(&content))?;
        }
        Ok(())
    }
}

impl ParseError {
    /// Should display with color or not, default: false
    pub fn with_color(self, colorful: bool) -> Self {
        ParseError { colorful, ..self }
    }
}
