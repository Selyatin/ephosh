use termion::{
    raw::IntoRawMode,
    event::Key,
};
use tui::{
    Terminal,
    backend::TermionBackend,
    style::{Style, Color, Modifier},
    text::{Spans, Span},
    widgets::{Block, Borders, Paragraph},
    layout::{Layout, Constraint, Direction},
};
use std::{
    io,
    env,
    path::Path,
    collections::HashMap
};
use config::Config;

mod ui;
mod utils;
mod event;
mod non_blocking;
mod inbuilt;
mod config;

use ui::Pane;

struct Shell {
    pub username: String,
    pub current_dir: String,
    pub commands: HashMap<String, String>,
    pub panes: Vec<Pane>,
    pub error: String,
    pub input: String,
    pub config: Config
}

impl Default for Shell {
    fn default() -> Self {
        let commands = match utils::get_commands_from_path(){
            Ok(commands) => commands,
            Err(err) => panic!("Error: {}", err)
        };
        let home_var = match env::var("HOME") {
            Ok(var) => var,
            Err(_) => panic!("Error: Couldn't get HOME var")
        };
        let config_path = format!("{}/.config/ephosh/ephosh.yml", home_var);

        let config = match Path::new(&config_path).is_file() {
            true => Config::new(config_path),
            false => Config::default()
        };

        let current_dir = std::env::current_dir().unwrap().to_str().unwrap().to_owned();

        let username = std::env::var("USER").unwrap().to_owned();

        Self {
            username,
            current_dir,
            commands,
            config,
            error: "".to_owned(),
            input: "".to_owned(),
            panes: vec![]        
        }
    }
}

fn main() -> Result<(), io::Error> {

    let mut shell = Shell::default();

    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let events = event::Events::new();

    // Clear the screen
    std::process::Command::new("clear").spawn().unwrap();

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([
                    Constraint::Percentage(95),
                    Constraint::Percentage(5),
                ].as_ref())
                .split(f.size());

            let block = Block::default()
                .title("Input")
                .borders(Borders::ALL);
            f.render_widget(block, chunks[1]);

            f.set_cursor(chunks[1].x + 1 + shell.input.len() as u16, chunks[1].y+1);

            let panes_len = match shell.panes.len() {
                0 => 1,
                _ => shell.panes.len()
            };

            let percentage = (100 / panes_len) as u16;

            let mut constraints: Vec<Constraint> = vec![];

            for _ in &shell.panes {
                let constraint = Constraint::Percentage(percentage);
                constraints.push(constraint);
            }

            let output_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(constraints)
                .split(chunks[0]);

            for (i, pane) in shell.panes.iter().enumerate() {
                f.render_widget(pane.get_output_as_paragraph(), output_layout[i]);
            }

            let username = Span::styled(format!("[ {} | ", &shell.username), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));
            let current_dir = Span::styled(format!("{} | ", &shell.current_dir), Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD));
            let error = Span::styled(format!("{}", &shell.error), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));

            let status_info = Spans::from(vec![username, current_dir, error, Span::raw(" ]")]);

            let status_line = Paragraph::new(shell.input.as_ref())
                .block(Block::default()
                    .borders(Borders::ALL).title(status_info));

            f.render_widget(status_line, chunks[1]);
        }).unwrap();

        if let event::Event::Input(input) = events.next().unwrap() {
            match input {
                Key::Char('\n') => {
                    let args: Vec<&str> = shell.input.split_whitespace().collect();

                    if args.is_empty(){
                        continue;
                    }

                    match args[0] {
                        "cd" => {
                            if args.len() > 1 {
                                match inbuilt::cd(args[1]){
                                    Ok(_) => shell.current_dir = std::env::current_dir().unwrap().to_str().unwrap().to_owned(),

                                    Err(err) => shell.error = err.to_string()
                                };
                            };
                            shell.input.clear();
                            continue;
                        }
                        "clear" => {
                            if args.len() < 2 {
                                shell.error.clear();
                                shell.panes.clear();
                                shell.input.clear();
                                continue;
                            }

                            let index = match args[1].parse::<usize>(){
                                Ok(value) => value,

                                Err(_) => {
                                    shell.error = "Incorrect arguments were provided".to_owned();
                                    shell.input.clear();
                                    continue;
                                }
                            };

                            let value = if index <= 1 {0} else {(index - 1) as usize};

                            shell.panes.remove(value);

                            shell.input.clear();
                            continue;
                        }

                        "reload" => {
                            shell.commands = utils::get_commands_from_path().unwrap();
                            shell.input.clear();
                            continue;
                        },

                        "exshell.input" => break,

                        _ => {}
                    }

                    let path_var_result = env::var("PATH");

                    if let Err(_err) = path_var_result {
                        if shell.panes.len() < shell.config.max_outputs {
                            //shell.panes.push(err.to_string());
                            shell.input.clear();
                        }
                        continue;
                    }

                    let cmd = match shell.commands.get(args[0]){
                        Some(cmd) => cmd,
                        None => {
                            shell.input.clear();
                            continue;
                        }
                    };

                    let command_result = non_blocking::Command::new(&cmd).args(&args[1..]).spawn();

                    if let Err(_err) = command_result {
                        if shell.panes.len() < shell.config.max_outputs { 
                            shell.input.clear();
                        }
                        continue;
                    }

                    let (sender, receiver) = command_result.unwrap();

                    let pane = Pane::new(sender, receiver);

                    if shell.panes.len() >= shell.config.max_outputs {
                        shell.panes.remove(0);
                    }
                    shell.panes.push(pane);
                    shell.input.clear();
                }
                Key::Char('q') => {
                    break;
                }
                Key::Char(c) => {
                    shell.input.push(c);
                }
                Key::Backspace => {
                    shell.input.pop();
                },
                _ => {}
            }
        }

        for pane in shell.panes.iter_mut() {
            pane.recv();
        }
    }
    Ok(())
}
