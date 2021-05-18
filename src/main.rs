use std::{
    cmp::Ordering,
    env,
    io::{self, Write},
};
use crossterm::terminal;

mod config;
mod inbuilt;
mod shell;
mod utils;

use shell::Shell;

fn main(){
    let mut shell = Shell::default();
    let mut stdout = io::stdout();
    
    let mut child = shell::Command::new(&shell.pty, vec!["vim"], shell.terminal_size).unwrap();

    loop {
        stdout.write(child.get_output().unwrap());
        stdout.flush().unwrap();
    }
}
