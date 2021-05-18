use std::{
    cmp::Ordering,
    env,
    io::{self, Write},
};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use ephosh::event::*;

mod config;
mod inbuilt;
mod shell;
mod utils;

use shell::Shell;

fn main(){
    let mut shell = Shell::default();
    let mut stdout = io::stdout();

    enable_raw_mode().unwrap();
    execute!(stdout, EnterAlternateScreen).unwrap();
    
    let mut child = shell::Command::new(&shell.pty, vec!["vim"], shell.terminal_size).unwrap();

    let events = Events::new(250);

    let mut is_first_render = true;
    let mut current_buffer: Vec<u8> = vec![];

    loop {
        let output = child.get_output().unwrap();

        if current_buffer != output || is_first_render {
            stdout.write(output).unwrap();

            current_buffer = output.to_vec();

            is_first_render = false;
            stdout.flush().unwrap();
        }


        if let Event::Input(ev) = events.next().unwrap() {
            match ev {
                Key::Char(c) => {
                    match c {
                        'q' => shell.should_exit = true,
                        _ => ()
                    }
                }
                _ => ()
            }
        }

        if shell.should_exit {
            break;
        }
    }

    exit().unwrap();
}

fn exit() -> Result<(), Box<dyn std::error::Error>> {
    disable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, LeaveAlternateScreen, crossterm::cursor::Show)?;
    Ok(())
}
