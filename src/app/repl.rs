use crate::app::lox::Lox;
use crate::app::runner::run_source;
use std::io::Write;

pub(crate) fn run_prompt(lox: &mut Lox) {
    loop {
        print!("> ");
        match std::io::stdout().flush() {
            Ok(()) => (),
            Err(error) => {
                eprintln!("Failed to flush stdout: {error}");
                std::process::exit(1);
            }
        }

        let mut line = String::new();
        let bytes_read = match std::io::stdin().read_line(&mut line) {
            Ok(bytes_read) => bytes_read,
            Err(error) => {
                eprintln!("Failed to read stdin: {error}");
                std::process::exit(1);
            }
        };
        if bytes_read == 0 {
            break;
        }
        if line == "\n" || line == "exit\n" {
            break;
        }

        run_source(lox, &line);
        lox.had_error = false;
        lox.had_runtime_error = false;
    }
}
