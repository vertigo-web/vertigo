#![allow(clippy::needless_collect)]
#![allow(clippy::collapsible_else_if)]

use std::env::current_dir;
use std::fs::{self, read_to_string};
use std::path::Path;
use colored::*;

const EXTENSIONS_IGNORED: [&str; 6] = ["webp", "jpeg", "jpg", "png", "git", "wasm"];
const IGNORE_NAME: [&str; 1] = [".DS_Store"];

fn log_ok(message: impl Into<String>) {
    let message = message.into().green();
    println!("{message}");
}

fn log_error(message: impl Into<String>) {
    let message = message.into().red();
    println!("{message}");
}

fn visit_dirs(result: &mut Vec<String>, dir: &Path) {
    if dir.is_dir() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            visit_dirs(result, &path);
        }
        return;
    }

    if dir.is_file() {
        if dir.starts_with("./.git") ||
            dir.starts_with("./target") ||
            dir.starts_with("./lint-project") ||
            dir.starts_with("./demo/build") ||
            dir.starts_with("./.vscode") {
            //ignore
            return;
        }

        if let Some(ext) = dir.extension() {
            let ext_str = ext.to_str().unwrap();

            for pattern_ext in EXTENSIONS_IGNORED {
                if pattern_ext.trim() == ext_str.trim() {
                    return;
                }
            }
        }

        if let Some(name) = dir.file_name() {
            let name_str = name.to_str().unwrap();
            
            for pattern_ext in IGNORE_NAME {
                if pattern_ext.trim() == name_str.trim() {
                    return;
                }
            }
        }

        if let Some(dir) = dir.to_str() {
            result.push(dir.into());
        } else {
            log_error("Error with conversion of characters to utf8");
        }
        return;
    }

    log_error(format!("Unsupported type {dir:?}"));
}

fn test_file(errors_counter: &mut u64, file_path: &String, content: String) {
    if let Some(char) = content.chars().rev().next() {
        if char == '\n' {
            //ok
        } else {
            log_error(format!("file_path={file_path}"));
            log_error("Newline character missing at end of file");

            let last_chars = content.chars().rev().take(10).collect::<Vec<_>>();
            let last_chars = last_chars.into_iter().rev().collect::<Vec<_>>();
            log_error(format!("last_chars={last_chars:?}"));

            println!();

            *errors_counter += 1;
        }
    }

    for (line, text) in content.lines().enumerate() {
        if text.starts_with("///") {
            continue;
        }

        let line = line + 1;

        let text = text.to_lowercase();

        let todo_index1 = text.find("//");
        let todo_index2 = text.find("todo");

        if let (Some(todo_index1), Some(todo_index2)) = (todo_index1, todo_index2) {
            if todo_index1 < todo_index2 {
                log_error(format!("file_path={file_path}:{line}"));
                log_error("The line includes \"// todo\"");
                log_error(format!("line text: {text}"));
                println!();

                *errors_counter += 1;
            }
        }

        if text.contains("todo!") {
            log_error(format!("file_path={file_path}:{line}"));
            log_error("The line includes \"todo!\"");
            log_error(format!("line text: {text}"));
            println!();

            *errors_counter += 1;
        }

        if file_path.starts_with("./crates/vertigo-macro") || file_path == "./crates/vertigo/build.rs" {
            //ignore
        } else {
            if text.contains("unwrap(") {
                log_error(format!("file_path={file_path}:{line}"));
                log_error("The line includes \"unwrap(\"");
                log_error(format!("line text: {text}"));
                println!();

                *errors_counter += 1;
            }
        }

        if text.contains("unimplemented!(") {
            log_error(format!("file_path={file_path}:{line}"));
            log_error("The line includes \"unimplemented!(\"");
            log_error(format!("line text: {text}"));
            println!();

            *errors_counter += 1;
        }

        // println!("line={line} text={text}");
    }
}

fn main() {
    let current = current_dir().unwrap();
    println!("Linting current_dir={current:?} ...");

    let mut result = Vec::new();

    visit_dirs(&mut result, Path::new("."));

    let mut errors_counter = 0;

    for file_path in result {
        let content = read_to_string(&file_path);

        match content {
            Ok(content) => {
                test_file(&mut errors_counter, &file_path, content);
            },
            Err(error) => {
                log_error(format!("Error read file={file_path} error={error}"));
                return;
            }
        }
    }

    if errors_counter > 0 {
        log_error(format!("errors: {errors_counter}"));
        println!();
    } else {
        log_ok("lint ok");
        println!();
    }
}
