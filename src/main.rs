use std::io;
use termion::raw::IntoRawMode;
use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{Block, Borders, Paragraph, List, ListItem};
use tui::text::{Span, Spans};
use tui::layout::{Layout, Constraint, Direction};
use termion::event::Key;
use std::{
    env,
    sync::mpsc,
    process::Command,
    path::Path
};
mod event;
fn main() -> Result<(), io::Error> {
    let (tx, rx) = mpsc::channel();
    let mut cmds: Vec<String> = Vec::new();
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
                .title("Output 1")
                .borders(Borders::ALL);
            f.render_widget(block, chunks[0]);
                
            let block = Block::default()
                .title("Input")
                .borders(Borders::ALL);
            f.render_widget(block, chunks[1]);
            
            f.set_cursor(chunks[1].x + 1 + it.len() as u16, chunks[1].y+1);
            
            let messages: Vec<ListItem> = cmds
                .iter()
                .map(|o|{
                    let content = vec![Spans::from(Span::raw(format!("{}", o)))];
                    ListItem::new(content)
                })
                .collect();
                
            let messages = List::new(messages)
                .block(Block::default()
                .borders(Borders::ALL).title("Output"));
                
            f.render_widget(messages, chunks[0]);
                
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
                        break;
                    }
                    
                    let path_var_result = env::var("PATH");
                    
                    if let Err(_) = path_var_result {
                        break;
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
                    
                    let mut command = Command::new(&cmd);
                    
                    for arg in args[1..].iter() {
                        command.arg(arg);
                    }
                    
                    let output_result: String;
                    
                    if let Err(err) = command.output() {
                        output_result = err.to_string();
                    } else {
                        output_result = String::from_utf8(command.output().unwrap().stdout).unwrap();
                    }
                    
                    
                    if let Err(_) = tx.send(output_result){
                        break;
                    }
                    
                    if let Ok(data) = rx.try_recv(){
                        cmds.push(format!("{}: {}", args[0], data));
                    }
                    
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
    }
    Ok(())
}
