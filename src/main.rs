use crate::scanner::Scanner;
use clap::Parser;
use std::fs::File;
use std::io::{BufReader, Read, Write};

pub mod scanner;
pub mod token;
#[derive(Debug, Parser)]
struct Args {
    source_path: Option<String>,
}

struct Lox {
    had_error: bool,
}

impl Lox {
    fn new() -> Self {
        Lox { had_error: false }
    }

    pub(crate) fn report_error(&mut self, line: usize, message: &str) {
        self.had_error = true;
        eprintln!("[line {}] Error: {}", line, message);
    }

    fn run_prompt(&mut self) {
        loop {
            print!("> ");
            std::io::stdout().flush().unwrap();
            let mut line = String::new();
            std::io::stdin().read_line(&mut line).unwrap();
            if line == "\n" || line == "exit\n" {
                break;
            }
            self.run(line);
        }
    }

    fn run_file(&mut self, path: &str) {
        let file = File::open(path).unwrap();
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents).unwrap();
        self.run(contents);
    }

    fn run(&mut self, source: String) {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens(self);
        println!("{:?}", tokens);
    }
}

fn main() {
    let mut lox = Lox::new();
    let args = Args::parse();
    if let Some(path) = args.source_path {
        lox.run_file(&path);
    } else {
        lox.run_prompt();
    }
    if lox.had_error {
        std::process::exit(65);
    }
}
