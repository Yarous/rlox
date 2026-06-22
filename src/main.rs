use crate::run::{run_file, run_prompt};

mod run;
pub mod scanner;
pub mod token;


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

    pub(crate) fn run(&mut self) {
        run_file();
    }
    fn run_prompt(&mut self) {
        run_prompt();
        self.had_error = false;
    }
}

fn main() {
    let lox = Lox::new();
    lox.run();
    if lox.had_error {
        std::process::exit(65);
    }
}
