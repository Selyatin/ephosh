use std::io;
use termion::raw::IntoRawMode;
use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{Block, Borders, Paragraph, List, ListItem, Table, Row};
use tui::text::{Span, Spans};
use tui::layout::{Layout, Constraint, Direction};
use termion::event::Key;
use std::{
    env,
    sync::mpsc::{self, Sender, Receiver, TryRecvError},
    path::Path
};

mod event;
mod non_blocking;

fn main() -> Result<(), io::Error> {
    let mut output_pane: Vec<String> = vec![];

    let mut output_receivers: Vec<Receiver<String>> = vec![];

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
            

            let mut output_newlines: Vec<Vec<&str>> = vec![];
            
            for output in &output_pane {
                let newlines: Vec<&str> = output.split("\n").collect();
                output_newlines.push(newlines);
            }
            
            let mut rows: Vec<Row> = vec![];
            
            output_newlines.sort_by(|a, b| {
                b.len().cmp(&a.len())
            });
            
            let mut i = 0;

            let length = output_newlines.get(0).unwrap_or(&Vec::new()).len();

            while i < length {
                let mut row_lines: Vec<&str> = vec![];
                
                for lines in &output_newlines {
                    
                    match lines.get(i) {
                        Some(line) => row_lines.push(line),
                        None => {}
                    };
                }
                
                let row = Row::new(row_lines);

                rows.push(row);

                i += 1;
            }

            let mut widths: Vec<Constraint> = vec![];
            
            let output_pane_len = match output_pane.len() {
                0 => 1,
                _ => output_pane.len()
            };
            let percentage = (100 / output_pane_len) as u16;
            for _ in 0..output_pane_len - 1 {
                let constraint = Constraint::Percentage(percentage);
                widths.push(constraint);
            }

            let table = Table::new(rows).block(Block::default().title("Table").borders(Borders::ALL))
                .widths(&widths);
            f.render_widget(table, chunks[0]);
            
            drop(widths);
            drop(percentage);
            drop(output_pane_len);
            drop(length);
            drop(output_newlines);
            
            let inp = Paragraph::new(it.as_ref())
                .block(Block::default()
                .borders(Borders::ALL).title("Input"));
            f.render_widget(inp, chunks[1]);
        }).unwrap();
    
        if let event::Event::Input(input) = events.next().unwrap() {
            match input {
                Key::Char('\n') => {
                    let args: Vec<&str> = it.split_whitespace().collect();
                    let mut cmd = "".to_owned();
                                        
                    if args.is_empty(){
                        continue;
                    }
                    
                    if args[0] == "clear" {
                        output_pane.clear();
                        it.clear();
                        continue;
                    }

                    if args[0] == "exit" {
                        break;
                    }
                    
                    let path_var_result = env::var("PATH");
                    
                    if let Err(err) = path_var_result {
                        output_pane.push(err.to_string());
                        continue;
                    }
                    
                    let path_var = path_var_result.unwrap();
                    
                    let mut paths_vec: Vec<String> = path_var.split(":")
                        .collect::<Vec<&str>>()
                        .into_iter()
                        .map(|s| s.to_owned()).collect();
                    
                    for path in paths_vec.iter_mut() {
                        path.push_str("/");
                        path.push_str(args[0]);
                        
                        if Path::new(&path).exists() {
                            cmd = path.to_string();
                        }
                    }
                    
                    let command_result = non_blocking::Command::new(&cmd).args(&args[1..]).spawn();
                    
                    if let Err(err) = command_result {
                        output_pane.push(err.to_string());
                        continue;
                    }
                    
                    let (_sender, receiver) = command_result.unwrap();

                    output_receivers.push(receiver);

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
        
        let mut i: usize = 0;
        while i < output_receivers.len(){
            let receiver = &output_receivers[i];

            match receiver.try_recv() {
                Ok(message) => output_pane.push(message),

                Err(err) => {
                    if err == TryRecvError::Disconnected {
                        output_receivers.remove(i);
                    }
                }
            };

            i += 1;
        }
    }
    Ok(())
}
