use std::env;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use zero_shell::commands::*;
use zero_shell::parse::parse_line;

fn flush_stdout() {
    if let Err(e) = io::stdout().flush() {
        eprintln!("zero-shell: flush error: {}", e);
    }
}

fn main() {
    let _ = ctrlc::set_handler(|| {
        eprintln!();
    });

    let mut current_dir = match env::current_dir() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("zero-shell: could not read current directory: {}", e);
            PathBuf::from("/")
        }
    };

    loop {
        print!("$ ");
        flush_stdout();

        let stdin = io::stdin();
        let mut line = String::new();

        match stdin.lock().read_line(&mut line) {
            Ok(0) => {
                println!();
                break;
            }
            Ok(_) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                let parts = match parse_line(line) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("zero-shell: {}", e);
                        continue;
                    }
                };

                if parts.is_empty() {
                    continue;
                }

                let command = parts[0].as_str();
                let args: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();

                match command {
                    "echo" => echo(&args),
                    "cd" => {
                        if let Some(new_dir) = cd(&args, &current_dir) {
                            current_dir = new_dir;
                        }
                    }
                    "pwd" => pwd(&current_dir),
                    "ls" => ls(&args, &current_dir),
                    "cat" => cat(&args, &current_dir),
                    "cp" => cp(&args, &current_dir),
                    "rm" => rm(&args, &current_dir),
                    "mv" => mv(&args, &current_dir),
                    "mkdir" => mkdir(&args, &current_dir),
                    "exit" => break,
                    _ => {
                        eprintln!("Command '{}' not found", command);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }
}
