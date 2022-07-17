use backtrace::Backtrace;
use manifest_dir_macros::*;
use pretty_assertions::StrComparison;
use scheme_parser::*;
use std::ffi::OsStr;
use std::fs::{self, remove_file, File};
use std::io::{self, Write};
use std::path::Path;
use walkdir::WalkDir;

mod span_to_source;
use span_to_source::*;

fn read_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let s = fs::read_to_string(path)?;
    Ok(s.replace("\r\n", "\n"))
}
fn path_to_string(path: &Path) -> String {
    path.to_str().unwrap().to_string().replace('\\', "/")
}

#[derive(Default)]
struct RegressionError {
    errors: Option<String>,
}

impl RegressionError {
    fn insert(&mut self, mut f: impl FnMut(&mut String)) {
        f(self.errors.get_or_insert_with(Default::default));
        let current_backtrace = Backtrace::new();
        let s = self.errors.get_or_insert_with(Default::default);
        use std::fmt::Write;
        writeln!(s, "Backtrace: {:?}", current_backtrace).unwrap();
    }
    fn expect_ok(&self) {
        if let Some(errors) = &self.errors {
            panic!("{}", errors)
        }
    }
}

fn assert_eq_or_override(path: &Path, actual: &str, regression_error: &mut RegressionError) {
    if cfg!(feature = "override-test") {
        let mut file = File::create(path).unwrap();
        file.write_all(actual.as_bytes()).unwrap();
    }
    let expected = read_to_string(path).unwrap_or_else(|_| String::new());
    if expected != actual {
        regression_error.insert(|s| {
            use std::fmt::Write;
            writeln!(s, "{}", StrComparison::new(&expected, actual)).unwrap();
        })
    }
}

fn assert_non_exist(path: &Path, regression_error: &mut RegressionError) {
    if cfg!(feature = "override-test") && path.exists() {
        remove_file(path).unwrap();
    }
    if path.exists() {
        regression_error.insert(|s| {
            use std::fmt::Write;
            writeln!(s, "{} should not exists", path.display()).unwrap();
        })
    }
}

#[test]
fn regression() {
    let dir = path!("tests/");
    let mut regression_errors = Default::default();
    for entry in WalkDir::new(dir) {
        let path = entry.unwrap().path().to_owned();
        if path.extension() != Some(OsStr::new("scm")) {
            continue;
        }
        let path_str = path_to_string(path.strip_prefix(path!(".")).unwrap());
        let input = read_to_string(&path).unwrap();
        {
            let mut error_path = path.clone();
            error_path.set_extension("token.err");
            let mut token_path = path.clone();
            token_path.set_extension("token");
            let (result, error) = tokenize(&input, &path_str);
            if let (Some(result), None) = (result, &error) {
                let content = format!("{:#?}", SpanToSource(&result, &input));
                assert_eq_or_override(&token_path, &content, &mut regression_errors);
            } else {
                assert_non_exist(&token_path, &mut regression_errors);
            }
            if let Some(error) = error {
                let content = error.to_string();
                assert_eq_or_override(&error_path, &content, &mut regression_errors);
            } else {
                assert_non_exist(&error_path, &mut regression_errors);
            }
        }
        {
            let mut error_path = path.clone();
            error_path.set_extension("ast.err");
            let mut ast_path = path.clone();
            ast_path.set_extension("ast");
            let (result, error) = parse(&input, &path_str);
            if let (Some(result), None) = (result, &error) {
                let content = format!("{:#?}", SpanToSource(&result, &input));
                assert_eq_or_override(&ast_path, &content, &mut regression_errors);
            } else {
                assert_non_exist(&ast_path, &mut regression_errors);
            }
            if let Some(error) = error {
                let content = error.to_string();
                assert_eq_or_override(&error_path, &content, &mut regression_errors);
            } else {
                assert_non_exist(&error_path, &mut regression_errors);
            }
        }
    }
    regression_errors.expect_ok();
}
