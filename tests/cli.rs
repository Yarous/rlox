use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

fn binary() -> String {
    std::env::var("CARGO_BIN_EXE_rlox").unwrap()
}

fn source_file(source: &str) -> std::path::PathBuf {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let path =
        std::env::temp_dir().join(format!("rlox_cli_test_{}_{}.lox", std::process::id(), id));
    fs::write(&path, source).unwrap();
    path
}

fn run_file(source: &str) -> std::process::Output {
    let path = source_file(source);
    let output = Command::new(binary()).arg(&path).output().unwrap();
    fs::remove_file(path).unwrap();
    output
}

fn run_prompt(input: &str) -> std::process::Output {
    let mut child = Command::new(binary())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();

    child.wait_with_output().unwrap()
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn file_prints_arithmetic_result() {
    let output = run_file("print 1 + 2 * 3;\n");

    assert!(output.status.success());
    assert_eq!(stdout(&output), "7\n");
    assert_eq!(stderr(&output), "");
}

#[test]
fn file_runs_variables_blocks_and_assignment() {
    let output = run_file("var a = 1;\n{ a = a + 2; }\nprint a;\n");

    assert!(output.status.success());
    assert_eq!(stdout(&output), "3\n");
}

#[test]
fn file_runs_functions() {
    let output = run_file("fun add(a, b) { return a + b; }\nprint add(2, 3);\n");

    assert!(output.status.success());
    assert_eq!(stdout(&output), "5\n");
}

#[test]
fn file_runs_closures() {
    let output = run_file(
        "fun make() { var x = 10; fun get() { return x; } return get; }\nvar get = make();\nprint get();\n",
    );

    assert!(output.status.success());
    assert_eq!(stdout(&output), "10\n");
}

#[test]
fn file_exits_65_on_scan_error() {
    let output = run_file("@\n");

    assert_eq!(output.status.code(), Some(65));
    assert!(stderr(&output).contains("Failed to parse lexeme as token"));
}

#[test]
fn file_exits_65_on_parse_error() {
    let output = run_file("break;\n");

    assert_eq!(output.status.code(), Some(65));
    assert!(stderr(&output).contains("Can't use 'break' outside of a loop."));
}

#[test]
fn file_exits_70_on_runtime_error() {
    let output = run_file("print 1 / 0;\n");

    assert_eq!(output.status.code(), Some(70));
    assert!(stderr(&output).contains("Division by Zero"));
}

#[test]
fn file_rejects_top_level_return() {
    let output = run_file("return 1;\nprint 2;\n");

    assert_eq!(output.status.code(), Some(65));
    assert_eq!(stdout(&output), "");
    assert!(stderr(&output).contains("return statement only allowed"));
}

#[test]
fn prompt_evaluates_print_and_exit() {
    let output = run_prompt("print 42;\nexit\n");

    assert!(output.status.success());
    assert_eq!(stdout(&output), "> 42\n> ");
    assert_eq!(stderr(&output), "");
}

#[test]
fn prompt_resets_errors_after_each_line() {
    let output = run_prompt("print missing;\nprint 2;\nexit\n");

    assert!(output.status.success());
    assert!(stdout(&output).ends_with("> 2\n> "));
    assert!(stderr(&output).contains("Undefined variable: missing"));
}
