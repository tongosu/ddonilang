use std::io::{self, Write};

use crate::cli::run;
use crate::core::hash;
use crate::core::{State, Trace};

pub fn repl() -> Result<(), String> {
    let mut state = State::new();
    let stdin = io::stdin();

    println!("또니랑 REPL (WALK01)");
    println!(":hash - 현재 state_hash 출력");
    println!(":quit - 종료");
    println!();

    loop {
        print!("또니랑> ");
        io::stdout().flush().map_err(|e| e.to_string())?;

        let mut line = String::new();
        if stdin.read_line(&mut line).map_err(|e| e.to_string())? == 0 {
            break;
        }

        let line = line.trim();
        if line == ":quit" || line == ":q" {
            break;
        }
        if line == ":hash" || line == ":h" {
            println!("state_hash={}", hash::state_hash(&state));
            continue;
        }
        if line.is_empty() {
            continue;
        }

        match run_line(&mut state, line) {
            Ok(trace) => {
                for log in trace.log_lines() {
                    println!("{}", log);
                }
            }
            Err(err) => {
                eprintln!("{}", err);
            }
        }
    }

    Ok(())
}

fn run_line(state: &mut State, line: &str) -> Result<Trace, String> {
    let output = run::run_source_with_state(line, state.clone()).map_err(|e| e.format("<repl>"))?;
    *state = output.state;
    Ok(output.trace)
}
