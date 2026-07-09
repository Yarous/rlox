use crate::app::lox::Lox;
use crate::app::repl::run_prompt;
use crate::app::runner::run_source;
use clap::Parser;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct Args {
    source_path: Option<PathBuf>,
}

pub fn run() {
    let mut lox = Lox::new();
    let args = Args::parse();

    if let Some(path) = args.source_path.as_deref() {
        if let Err(error) = run_file(&mut lox, path) {
            lox.report_error(0, "", &error.to_string());
        }
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

fn run_file(lox: &mut Lox, path: &std::path::Path) -> std::io::Result<()> {
    let contents = fs::read_to_string(path)?;
    run_source(lox, &contents);
    Ok(())
}
