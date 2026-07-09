use crate::app::lox::Lox;
use crate::app::runner::run_source;
use std::io::Write;

pub(crate) fn run_prompt(lox: &mut Lox) {
    loop {
        print!("> ");
        match std::io::stdout().flush() {
            Ok(_) => (),
            Err(error) => {
                eprintln!("Failed to flush stdout: {}", error);
                std::process::exit(1);
            }
        };

        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        if line == "\n" || line == "exit\n" {
            break;
        }

        run_source(lox, &line);
        lox.had_error = false;
        lox.had_runtime_error = false;
    }
}
