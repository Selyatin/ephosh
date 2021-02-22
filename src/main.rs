use std::io;
use termion::{
    raw::IntoRawMode,
    event::Key,
};
use tui::{
    Terminal,
    backend::TermionBackend,
    widgets::{Block, Borders, Paragraph},
    layout::{Layout, Constraint, Direction},
};
use std::{
    env,
    path::Path
};
use config::Config;
use clap::{
    App, 
    Arg,
    crate_version,
    crate_authors,
};

mod ui;
mod utils;
mod event;
mod non_blocking;
mod inbuilt;
mod config;

use ui::Pane;

fn main() -> Result<(), io::Error> {
    let args = App::new("ephosh")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(
            Arg::with_name("config")
            .help("Path to configuration file")
            .short("c") 
            .long("config") 
            .multiple(false)
            .takes_value(true),
        )
        .get_matches();

    let config: Config;

    let mut current_error = String::new();

    config = match args.occurrences_of("config") {
        0 => {
            let config_path = &format!("{}/.config/ephosh/ephosh.yml", std::env::var("HOME").unwrap())[..];
            match Path::new(config_path).is_file() {
                true => Config::new(config_path),
                false => Config::default(),
            }
        }
        _ => {
            match args.value_of("config") {
                Some(value) => Config::new(value),
                None => Config::default(),
            }
        }
    };
    
    let mut commands = utils::get_commands_from_path().unwrap();
    
    let mut output_panes: Vec<Pane> = vec![];

    std::process::Command::new("clear").spawn().unwrap();

    let mut it = String::from("");

    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let events = event::Events::new();

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Percentage(94),
                    Constraint::Percentage(6),
                ].as_ref())
                .split(f.size());

            let block = Block::default()
                .title("Input")
                .borders(Borders::ALL);
            f.render_widget(block, chunks[1]);

            f.set_cursor(chunks[1].x + 1 + it.len() as u16, chunks[1].y+1);

            let output_panes_len = match output_panes.len() {
                0 => 1,
                _ => output_panes.len()
            };

            let percentage = (100 / output_panes_len) as u16;

            let mut constraints: Vec<Constraint> = vec![];

            for _ in &output_panes {
                let constraint = Constraint::Percentage(percentage);
                constraints.push(constraint);
            }

            let output_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(constraints)
                .split(chunks[0]);

            for (i, pane) in output_panes.iter().enumerate() {
                f.render_widget(pane.get_output_as_paragraph(), output_layout[i]);
            }

            let status_line = Paragraph::new(it.as_ref())
                .block(Block::default()
                    .borders(Borders::ALL).title(format!(" [ {} | {} {}]", 
                            std::env::var("USER").unwrap(), 
                            std::env::current_dir().unwrap().to_str().unwrap(),
                            if !current_error.is_empty() {
                                format!("| {} ", current_error)
                            } else { String::from("") }
                    )));

            f.render_widget(status_line, chunks[1]);
        }).unwrap();

        if let event::Event::Input(input) = events.next().unwrap() {
            match input {
                Key::Char('\n') => {
                    current_error.clear();
                    let args: Vec<&str> = it.split_whitespace().collect();

                    if args.is_empty(){
                        continue;
                    }

                    match args[0] {
                        "cd" => {
                            if args.len() > 1 {
                                if let Err(err) = inbuilt::cd(args[1]) {
                                    current_error = err.to_string();
                                }
                            };
                            it.clear();
                            continue;
                        }
                        "clear" => {
                            if args.len() < 2 {
                                output_panes.clear();
                                it.clear();
                                continue;
                            }
                            
                            let index = match args[1].parse::<usize>(){
                                Ok(value) => value,

                                Err(_) => {
                                    current_error = "Incorrect arguments were provided".to_owned();
                                    it.clear();
                                    continue;
                                }
                            };

                            let value = if index <= 1 {0} else {(index - 1) as usize};

                            output_panes.remove(value);

                            it.clear();
                            continue;
                        }
                        
                        "reload" => {
                            commands = utils::get_commands_from_path().unwrap();
                            it.clear();
                            continue;
                        },

                        "exit" => break,

                        _ => {}
                    }

                    let path_var_result = env::var("PATH");

                    if let Err(_err) = path_var_result {
                        if output_panes.len() < config.max_outputs {
                            //output_panes.push(err.to_string());
                            it.clear();
                        }
                        continue;
                    }

                    let cmd = match commands.get(args[0]){
                        Some(cmd) => cmd,
                        None => {
                            it.clear();
                            continue;
                        }
                    };

                    let command_result = non_blocking::Command::new(&cmd).args(&args[1..]).spawn();

                    if let Err(_err) = command_result {
                        if output_panes.len() < config.max_outputs { 
                            it.clear();
                        }
                        continue;
                    }

                    let (sender, receiver) = command_result.unwrap();

                    let pane = Pane::new(sender, receiver);

                    if output_panes.len() >= config.max_outputs {
                        output_panes.remove(0);
                    }
                    output_panes.push(pane);
                    it.clear();
                }
                Key::Char('q') => {
                    break;
                }
                Key::Char(c) => {
                    it.push(c);
                }
                Key::Backspace => {
                    it.pop();
                },
                _ => {}
            }
        }

        for pane in output_panes.iter_mut() {
            pane.recv();
        }
    }
    Ok(())
}
