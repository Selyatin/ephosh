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
    io::{self, Write},
    env,
    cmp::Ordering,
};

mod ui;
mod utils;
mod event;
mod shell;
mod non_blocking;
mod inbuilt;
mod config;

use ui::Pane;
use shell::Shell;

fn main() -> Result<(), io::Error> {

    let mut shell = Shell::default();

    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let events = event::Events::new();

    // Clear the screen
    print!("{}", termion::clear::All);

    print!("{}", termion::cursor::SteadyBar);
    let mut history_index: usize = 0;
    let mut current_history: Vec<String> = std::fs::read_to_string(&shell.config.history_path).unwrap()
        .split("\n")
        .map(|elem| {
            let mut elem_string = String::from("\n");
            elem_string.push_str(elem);
            elem_string
        })
        .collect();

    current_history.retain(|elem| elem != "\n");

    loop {
        if current_history.len() >= shell.config.history_size {
            current_history.remove(0);
        }
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([
                    Constraint::Percentage(95),
                    Constraint::Percentage(5),
                ].as_ref())
                .split(f.size());

            shell.chunks = chunks;
            let block = Block::default()
                .title("Input")
                .borders(Borders::ALL);
            f.render_widget(block, shell.chunks[1]);

            f.set_cursor(shell.cursor.x, shell.chunks[1].y+1);

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
                .split(shell.chunks[0]);

            for (i, pane) in shell.panes.iter().enumerate() {
                f.render_widget(pane.get_output_as_paragraph(), output_layout[i]);
            }

            let separator = Span::raw(" | ");
            let username = Span::styled(&shell.username, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));
            let current_dir = Span::styled(&shell.current_dir, Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD));
            let error = Span::styled(&shell.error, Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));

            let status_info = Spans::from(vec![Span::raw("[ "), username, separator.to_owned(), current_dir, separator, error, Span::raw(" ]")]);

            let status_line = Paragraph::new(shell.input.as_ref())
                .block(Block::default()
                    .borders(Borders::ALL).title(status_info));

            f.render_widget(status_line, shell.chunks[1]);
        }).unwrap();

        if let event::Event::Input(input) = events.next().unwrap() {
            match input {
                Key::Char('\n') => {
                    let args: Vec<&str> = shell.input.split_whitespace().collect();

                    if args.is_empty() {
                        continue;
                    }

                    current_history.push(format!("\n{}", shell.input));
                    history_index = 0;
                    shell.cursor.x = 1;

                    match args[0] {
                        "cd" => {
                            if args.len() > 1 {
                                match inbuilt::cd(args[1]){
                                    Ok(_) => shell.current_dir = env::current_dir().unwrap().to_str().unwrap().to_owned(),

                                    Err(err) => shell.error = err.to_string()
                                };
                            };
                            shell.input.clear();
                            continue;
                        }
                        "clear" => {
                            if args.len() < 2 {
                                shell.error.clear();
                                for pane in &mut shell.panes {
                                    if let Err(err) = pane.kill_process(){
                                        shell.error = err;
                                    }
                                }
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

                            if let Err(err) = shell.panes[value].kill_process(){
                                shell.error = err;
                            }
                            shell.panes.remove(value);

                            shell.input.clear();
                            continue;
                        }
                        "send" => {
                            if args.len() < 3 {
                                shell.error = "send command accepts 2 parameters: position, message".to_owned();
                                shell.input.clear();
                                continue;
                            }

                            if shell.panes.len() <= 0 {
                                shell.error = "No panes to communicate with".to_owned();
                                shell.input.clear();
                                continue;
                            }

                            let index: usize = args[1].parse().unwrap_or(0);
                            
                            let message = args[2].to_owned();

                            if let Err(err) = shell.panes[index].send_line(message){
                                shell.error = err;
                                shell.input.clear();
                                continue;
                            }
                        },

                        "exit" => {
                            shell.history.write_all(current_history.join("").as_bytes()).unwrap();
                            break;
                        }

                        _ => {}
                    }

                    let command_result = non_blocking::Command::new(args[0]).args(&args[1..]).spawn();

                    if let Err(_err) = command_result {
                        if shell.panes.len() < shell.config.max_outputs { 
                            shell.input.clear();
                        }
                        continue;
                    }

                    let (sender, receiver) = command_result.unwrap();

                    let pane = Pane::new(sender, receiver);

                    if shell.panes.len() >= shell.config.max_outputs {
                        if let Err(err) = shell.panes[0].kill_process(){
                            shell.error = err;
                        }
                        shell.panes.remove(0);
                    }
                    shell.panes.push(pane);
                    shell.input.clear();
                }
                Key::Char('q') => {
                    shell.history.write_all(current_history.join("").as_bytes()).unwrap();
                    break;
                }
                Key::Char(c) => {
                    shell.input.insert((shell.cursor.x - 1) as usize, c);
                    shell.cursor.x += 1;
                }
                Key::Backspace => {
                    if shell.cursor.x - 1 > 0 {
                        shell.cursor.x -= 1;
                        shell.input.pop();
                    }
                },
                Key::Up => {
                    let mut history_cloned = current_history.clone();
                    history_cloned.reverse();

                    if current_history.len() > history_index {
                        let command = history_cloned[history_index].trim_end();
                        shell.input = String::from(command)
                            .replace("\n", "");
                        history_index += 1;
                        shell.cursor.x = command.len() as u16;
                    }
                }
                Key::Down => {
                    let mut history_cloned = current_history.clone();
                    history_cloned.reverse();

                    match history_index.cmp(&0) {
                        Ordering::Greater => {
                            if history_index - 1 > 0 {
                                let command = history_cloned[history_index - 1].trim_end();
                                shell.input = String::from(command)
                                    .replace("\n", "");
                                history_index -= 1;
                                shell.cursor.x = command.len() as u16;
                            } else {
                                history_index = 0;
                                shell.input = String::new();
                                shell.cursor.x = 1;
                            }
                        }
                        _ => {
                            shell.input = String::new();
                        }
                    }
                }  
                Key::Left => {
                    if shell.cursor.x - 1 > 0 {
                        shell.cursor.x -= 1;
                    }
                }
                Key::Right => {
                    if shell.cursor.x <= shell.input.len() as u16 {
                        shell.cursor.x += 1;
                    }
                }
                _ => {}
            }
        }

        for pane in shell.panes.iter_mut() {
            pane.recv();
        }
    }
    Ok(())
}
