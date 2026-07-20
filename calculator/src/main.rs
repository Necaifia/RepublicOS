use std::io::{self, BufRead, Write};

use calculator::Calculator;

fn main() {
    let calc = Calculator::new();
    println!("Calc REPL — type expressions like `2 + 3`, or `quit` to exit.");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("> ");
        stdout.flush().unwrap_or(());

        let mut line = String::new();
        if stdin.lock().read_line(&mut line).is_err() || line.trim().is_empty() {
            continue;
        }

        let input = line.trim();
        if input == "quit" || input == "exit" {
            break;
        }

        match calc.eval(input) {
            Ok(result) => println!("= {result}"),
            Err(e) => println!("Error: {e}"),
        }
    }
}
