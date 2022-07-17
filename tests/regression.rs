use manifest_dir_macros::*;
use pretty_assertions::assert_eq;
use scheme_parser::*;
use std::ffi::OsStr;
use std::fs::{self, read_dir, remove_file, File};
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

fn assert_non_exist(path: &Path) {
    if cfg!(feature = "override-test") && path.exists() {
        remove_file(path).unwrap();
    }
    assert!(!path.exists());
}

#[test]
fn regression() {
    let dir = path!("tests/");
    for entry in read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
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
            if let Some(error) = error {
                let content = error.to_string();
                assert_eq_or_override(&error_path, &content);
            } else {
                assert_non_exist(&error_path)
            }
            if let Some(result) = result {
                let content = format!("{:#?}", result);
                assert_eq_or_override(&token_path, &content);
            } else {
                assert_non_exist(&token_path)
            }
        }
        {
            let mut error_path = path.clone();
            error_path.set_extension("ast.err");
            let mut ast_path = path.clone();
            ast_path.set_extension("ast");
            let (result, error) = parse(&input, &path_str);
            if let Some(error) = error {
                let content = error.to_string();
                assert_eq_or_override(&error_path, &content);
            } else {
                assert_non_exist(&error_path)
            }
            if let Some(result) = result {
                let content = format!("{:#?}", result);
                assert_eq_or_override(&ast_path, &content);
            } else {
                assert_non_exist(&ast_path)
            }
        }
    }
}
