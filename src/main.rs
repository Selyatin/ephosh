use std::{
    cmp::Ordering,
    env,
    io::{self, Write},
};

mod config;
mod inbuilt;
mod shell;
mod utils;

use shell::Shell;

fn main(){
    let mut shell = Shell::default();
    let mut stdout = io::stdout();
    
    let child = shell::Command::new(&shell.pty, vec!["vim"], shell.terminal_size).unwrap();
    
    std::thread::sleep(std::time::Duration::from_millis(100));
    stdout.write(&child.get_output().unwrap());
}
