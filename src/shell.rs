use std::{
    io::{self, Stdout, Read, Write},
    time::Duration
};
use crossterm::{
    event::{KeyCode, KeyEvent, KeyModifiers, Event, poll, read},
    ExecutableCommand,
    terminal::{self, enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType}
};
use portable_pty::{
    PtySystem,
    native_pty_system,
    PtySize
};
use super::command::Command;

#[derive(Debug)]
enum InputMode {
    Normal,
    Interact
}

const V_LINE: &str = "│";
const H_LINE: &str = "─";
const NEWLINE: &str = "\n\r";

pub struct Shell {
    stdout: Stdout,
    input: String,
    commands: Vec<Command>,
    terminal_size: (u16, u16),
    error: String,
    pty: Box<dyn PtySystem>
}

impl Shell {
    pub fn new() -> Self {
        enable_raw_mode().expect("Couldn't enable raw mode");
        let terminal_size = terminal::size().expect("Couldn't get terminal size");

        Self {
            input: "".to_owned(),
            stdout: io::stdout(),
            commands: vec![],
            pty: native_pty_system(),
            error: "".to_owned(),
            terminal_size
        }
    }

    fn draw_empty_line(&mut self) -> Result<(), io::Error>{
        let (cols, _) = self.terminal_size;

        for col in 0..cols {
            self.stdout.write(&['\n' as u8])?;
        }

        Ok(())
    }

    fn draw_horizontal_line(&mut self) -> Result<(), io::Error>{
        let (cols, _) = self.terminal_size;

        for _ in 0..cols {
            self.stdout.write(H_LINE.as_bytes())?;
        }
        self.stdout.write(NEWLINE.as_bytes())?;

        Ok(())
    }

    fn draw_input_box(&mut self) -> Result<(), io::Error>{
        self.draw_horizontal_line()?;
        self.stdout.write(V_LINE.as_bytes())?;
        self.stdout.write(self.input.as_bytes())?;
        self.stdout.write(V_LINE.as_bytes())?;
        self.stdout.write(NEWLINE.as_bytes())?;
        self.draw_horizontal_line()?;
        Ok(())
    }

    fn get_input(&mut self){
        if let Ok(true) = poll(Duration::from_millis(2)) {
            match read().expect("Couldn't read input"){
                Event::Key(KeyEvent{code: KeyCode::Char(c), modifiers: KeyModifiers::NONE}) => {
                    self.input.push(c);
                },
                Event::Key(KeyEvent{code: KeyCode::Enter, modifiers}) => {
                                            let split: Vec<&str> = self.input.split(" ").collect();
                            let command = match Command::new(&self.pty, split, self.terminal_size){
                                Ok(command) => command,
                                Err(err) => {
                                    self.error = err;
                                    return;
                                }
                            };

                            self.commands.push(command);
                            self.input.clear();
                },
                Event::Key(KeyEvent{code: KeyCode::Backspace, modifiers}) => {
                    self.input.pop();
                },
                Event::Key(KeyEvent{code: KeyCode::Char(c), modifiers: KeyModifiers::CONTROL}) => {
                    if c == 'q' {
                        panic!("Break mothafukaaa");
                    }
                },
                _ => {}
            };
        }
    }

    pub fn run(&mut self) -> Result<(), io::Error>{
        loop {
            self.stdout.execute(Clear(ClearType::All)).expect("Couldn't clear terminal");;
            for command in &mut self.commands {
                let output = match command.get_output() {
                    Ok(output) => output,
                    Err(err) => {
                        self.error = err;
                        continue;
                    }
                };

                self.stdout.write(output)?;
            }

            self.draw_empty_line()?;

            self.draw_input_box()?;

            self.stdout.flush()?;

            self.get_input();
        }
    }
}
