use crate::*;
use manifest_dir_macros::*;
use pretty_assertions::assert_eq;
use std::ffi::OsStr;
use std::fs::{self, read_dir, File};
use std::io::{self, Write};
use std::path::Path;
fn read_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let s = fs::read_to_string(path)?;
    Ok(s.replace("\r\n", "\n"))
}
fn path_to_string(path: &Path) -> String {
    path.to_str().unwrap().to_string().replace('\\', "/")
}
fn assert_eq_or_override(path: &Path, actual: &str) {
    if cfg!(feature = "override-test") {
        let mut file = File::create(path).unwrap();
        file.write_all(actual.as_bytes()).unwrap();
    }
    let expected = read_to_string(path).unwrap_or_else(|_| String::new());
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
    let (result, error) = parse_recover(&input, path);
    if let Some(error) = error {
        panic!("{}", error.with_color(true))
    }
    let content = format!("{:#?}", result.unwrap());
    assert_eq_or_override(Path::new(path!("tests/main.ast")), &content);
}
#[test]
fn tokenize_error_works() {
    let dir = path!("tests/tokenize/error");
    for entry in read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension() != Some(OsStr::new("scm")) {
            continue;
        }
        let path_str = path_to_string(path.strip_prefix(path!(".")).unwrap());
        let input = read_to_string(&path).unwrap();
        let (_, error) = tokenize(&input, &path_str);
        let content = error.unwrap().to_string();
        let mut error_path = path.clone();
        error_path.set_extension("err");
        assert_eq_or_override(&error_path, &content);
    }
}
#[test]
fn parse_error_works() {
    let dir = path!("tests/parse/error");
    for entry in read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension() != Some(OsStr::new("scm")) {
            continue;
        }
        let path_str = path_to_string(path.strip_prefix(path!(".")).unwrap());
        let input = read_to_string(&path).unwrap();
        let (_, error) = parse_recover(&input, &path_str);
        let content = error.unwrap().to_string();
        let mut error_path = path.clone();
        error_path.set_extension("err");
        assert_eq_or_override(&error_path, &content);
    }
}
