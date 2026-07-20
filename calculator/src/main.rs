use rustyline::{error::ReadlineError, Editor};

use calculator::Calculator;

fn main() {
    let calc = Calculator::new();
    let history_file = "calc_history.txt";

    let mut rl = Editor::<()>::new();
    if rl.load_history(history_file).is_err() {
        // first run — no history yet
    }

    println!("Calc REPL — type expressions like `2 + 3`, or `quit` to exit.");

    loop {
        match rl.readline("> ") {
            Ok(line) => {
                let input = line.trim().to_string();
                if input.is_empty() {
                    continue;
                }
                if input == "quit" || input == "exit" {
                    break;
                }

                rl.add_history_entry(&input);

                match calc.eval(&input) {
                    Ok(result) => {
                        // Format with context: show what was evaluated
                        println!("  {input} = {result}");
                    }
                    Err(e) => {
                        println!("  Error evaluating `{input}`: {e}");
                    }
                }
            }
            Err(ReadlineError::Eof) | Err(ReadlineError::Interrupted) => {
                println!();
                break;
            }
            Err(err) => {
                eprintln!("REPL error: {err}");
                break;
            }
        }
    }

    rl.save_history(history_file).unwrap_or(());
    println!("Bye!");
}
