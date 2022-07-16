use crate::*;
use manifest_dir_macros::*;
use pretty_assertions::assert_eq;
use std::ffi::OsStr;
use std::fs::{read_dir, read_to_string, File};
use std::io::Write;
use std::path::Path;
fn assert_eq_or_override(path: &Path, actual: &str) {
    if cfg!(feature = "override-test") {
        let mut file = File::create(path).unwrap();
        file.write_all(actual.as_bytes()).unwrap();
    }
    let expected = read_to_string(path)
        .unwrap_or_else(|_| String::new())
        .replace('\r', "");
    assert_eq!(&expected, actual)
}
#[test]
fn lexer_works() {
    let path = path!("tests/main.scm");
    let input = read_to_string(path).unwrap();
    let (result, error) = tokenize(&input, path);
    if let Some(error) = error {
        panic!("{}", error.with_color(true))
    }
    let content = format!("{:#?}", result.unwrap());
    assert_eq_or_override(Path::new(path!("tests/main.token")), &content);
}
#[test]
fn parser_works() {
    let path = path!("tests/main.scm");
    let input = read_to_string(path).unwrap();
    let result = parse(&input, path).unwrap_or_else(|e| panic!("{}", e.with_color(true)));
    let content = format!("{:#?}", result);
    assert_eq_or_override(Path::new(path!("tests/main.ast")), &content);
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
        let result = parse(&input, &path_str).unwrap_err();
        let content = result.to_string();
        let mut error_path = path.clone();
        error_path.set_extension("err");
        assert_eq_or_override(&error_path, &content);
    }
}
