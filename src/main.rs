use std::env;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

mod commands;

use commands::*;

fn main() {
    let mut current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
    
    loop {
        // Display prompt
        print!("$ ");
        io::stdout().flush().unwrap();
        
        // Read input
        let stdin = io::stdin();
        let mut line = String::new();
        
        match stdin.lock().read_line(&mut line) {
            Ok(0) => {
                // EOF (Ctrl+D)
                println!();
                break;
            }
            Ok(_) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                
                // Parse command
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.is_empty() {
                    continue;
                }
                
                let command = parts[0];
                let args = &parts[1..];
                
                // Execute command
                match command {
                    "echo" => echo(args),
                    "cd" => {
                        if let Some(new_dir) = cd(args, &current_dir) {
                            current_dir = new_dir;
                        }
                    }
                    "pwd" => pwd(&current_dir),
                    "ls" => ls(args, &current_dir),
                    "cat" => cat(args, &current_dir),
                    "cp" => cp(args, &current_dir),
                    "rm" => rm(args, &current_dir),
                    "mv" => mv(args, &current_dir),
                    "mkdir" => mkdir(args, &current_dir),
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
