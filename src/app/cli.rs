use crate::app::lox::Lox;
use crate::app::repl::run_prompt;
use crate::app::runner::run_source;
use clap::Parser;
use std::fs::File;
use std::io::{BufReader, Read};

#[derive(Debug, Parser)]
struct Args {
    source_path: Option<String>,
}

pub fn run() {
    let mut lox = Lox::new();
    let args = Args::parse();

    if let Some(path) = args.source_path {
        run_file(&mut lox, &path);
    } else {
        run_prompt(&mut lox);
    }

    if lox.had_runtime_error {
        std::process::exit(70);
    }
    if lox.had_error {
        std::process::exit(65);
    }
}

fn run_file(lox: &mut Lox, path: &str) {
    let file = File::open(path).unwrap();
    let mut reader = BufReader::new(file);
    let mut contents = String::new();
    reader.read_to_string(&mut contents).unwrap();
    run_source(lox, &contents);
}
