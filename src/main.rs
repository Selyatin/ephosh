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
mod shell;
mod non_blocking;
mod inbuilt;
mod config;

use ui::{Pane, input::{self, InputMode}};
use shell::Shell;

fn main() -> Result<(), io::Error> {

    let mut shell = Shell::default();

    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let events = input::Events::new();

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

            f.set_cursor(shell.cursor.get_x(), shell.chunks[1].y + 1);

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

            for (i, pane) in shell.panes.iter_mut().enumerate() {
                f.render_widget(pane.get_output_as_paragraph(), output_layout[i]);
            }

            let separator = Span::raw(" | ");
            let username = Span::styled(&shell.username, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));
            let current_dir = Span::styled(&shell.current_dir, Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD));

            let input_mode = match shell.input_mode {
                InputMode::Command => "Mode: Command",
                InputMode::Interact => "Mode: Interact"
            };

            let input_mode_span = Span::styled(input_mode, Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD));
            let active_pane = Span::styled(format!("Interacting With: {}", shell.active_pane), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
            let error = Span::styled(&shell.error, Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));

            let status_info = Spans::from(vec![
                Span::raw("[ "), 
                username, 
                separator.clone(), 
                current_dir, 
                separator.clone(), 
                input_mode_span,
                separator.clone(),
                active_pane,
                separator,
                error, 
                Span::raw(" ]")]);

            let status_line = Paragraph::new(shell.input.as_ref())
                .block(Block::default()
                    .borders(Borders::ALL).title(status_info));

            f.render_widget(status_line, shell.chunks[1]);
        }).unwrap();

        if let input::Event::Input(input) = events.next().unwrap() {
            let panes_len = shell.panes.len();
            if shell.input_mode == InputMode::Interact && panes_len > 0{
                match input {
                    Key::Char(c) => {
                        let index = shell.active_pane;

                        let pane = &mut shell.panes[index];

                        match pane.send(c){
                            Ok(_) => {},
                            Err(err) => shell.error = err
                        };
                    },

                    Key::Ctrl(c) => {
                        match c {
                            'e' => shell.input_mode = InputMode::Command,
                            'a' => {
                                let active_pane_index = shell.active_pane - 1; 

                                if active_pane_index < panes_len {
                                    shell.active_pane = active_pane_index;
                                }
                            },
                            'd' => {
                                let active_pane_index = shell.active_pane + 1;

                                if active_pane_index < panes_len {
                                    shell.active_pane = active_pane_index;
                                }
                            },
                            _ => {}
                        };
                    }

                    _ => {}
                };
            }else{
                match input {
                    Key::Char('\n') => {
                        let args: Vec<&str> = shell.input.split_whitespace().collect();

                        if args.is_empty() {
                            continue;
                        }

                        current_history.push(format!("\n{}", shell.input));
                        history_index = 0;
                        shell.cursor.move_cursor(1, shell.cursor.get_y());

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
                                        pane.kill_process();
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
                                
                                shell.panes[value].kill_process();
                                shell.panes.remove(value);

                                shell.input.clear();
                                continue;
                            }

                            "exit" => {
                                shell.history.write_all(current_history.join("").as_bytes()).unwrap();
                                break;
                            }

                            _ => {}
                        }
                        
                        let mut command = non_blocking::Command::new(args[0]);

                        command.args(&args[1..]);
                        
                        if let Err(err) = command.spawn(){
                            shell.error = err;
                            shell.input.clear();
                            continue;
                        }

                        let pane = Pane::new(command);

                        if shell.panes.len() >= shell.config.max_outputs {
                            shell.panes[0].kill_process();
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
                        shell.input.insert((shell.cursor.get_x() - 1) as usize, c);
                        shell.cursor.move_right();
                    }

                    Key::Backspace => {
                        if shell.cursor.get_x() - 1 > 0 {
                            shell.cursor.move_left();
                            shell.input.remove((shell.cursor.get_x() - 1) as usize);
                        }
                    }

                    Key::Up => {
                        let mut history_cloned = current_history.clone();
                        history_cloned.reverse();

                        if current_history.len() > history_index {
                            let command = history_cloned[history_index].trim_end();
                            shell.input = String::from(command)
                                .replace("\n", "");
                            history_index += 1;
                            shell.cursor.move_cursor(command.len() as u16, shell.cursor.get_y());
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
                                    shell.cursor.move_cursor(command.len() as u16, shell.cursor.get_y());
                                } else {
                                    history_index = 0;
                                    shell.input = String::new();
                                    shell.cursor.move_cursor(1, shell.cursor.get_y());
                                }
                            }
                            _ => {
                                shell.input = String::new();
                            }
                        }
                    }

                    Key::Left => {
                        if shell.cursor.get_x() - 1 > 0 {
                            shell.cursor.move_left();
                        }
                    }

                    Key::Right => {
                        if shell.cursor.get_x() <= shell.input.len() as u16 {
                            shell.cursor.move_right();
                        }
                    }

                    Key::Ctrl(c) => {
                        if c == 'e'{
                            shell.input_mode = InputMode::Interact;
                        }
                    }
                    _ => {}
                }
            }
        };
    }
    Ok(())
}
