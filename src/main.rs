use std::io;
use termion::raw::IntoRawMode;
use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{Block, Borders, Paragraph, List, ListItem};
use tui::text::{Span, Spans};
use tui::layout::{Layout, Constraint, Direction};
use termion::event::Key;
use std::sync::mpsc;
use std::env;
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

            let messages =
                List::new(messages).block(Block::default().borders(Borders::ALL).title("Output"));
            f.render_widget(messages, chunks[0]);

            let inp = Paragraph::new(it.as_ref()).block(Block::default().borders(Borders::ALL).title("Input"));
            f.render_widget(inp, chunks[1]);
            if let event::Event::Input(input) = events.next().unwrap() {
                match input {
                    Key::Char('\n') => {
                        let args = it.split_whitespace().collect::<Vec<&str>>();
                        let mut cmd = String::from("");
                        if !args.is_empty() {
                            let path = env::var("PATH").unwrap();
                            let paths = path.split(":").collect::<Vec<&str>>();
                            for path in paths {
                                let mut a = path.to_owned();
                                a.push('/');
                                a.push_str(args[0]);
                                if std::path::Path::new(&a[..]).exists() {
                                    cmd = a;
                                }
                            }
                            let mut op = std::process::Command::new(&cmd);
                            if !cmd.is_empty() {
                                for i in args[1..].iter() {
                                    op.arg(i);
                                }
                                let tx1 = mpsc::Sender::clone(&tx);
                                std::thread::spawn(move || {
                                    tx1.send(op.output().unwrap().stdout).unwrap();
                                });
                                cmds.push(format!("{}: {}", args[0], std::str::from_utf8(&(rx.recv().unwrap()[..]))
                                    .unwrap()));
                            }
                        }
                        it.clear();
                    }
                    Key::Char('q') => std::process::exit(0),
                    Key::Char(c) => {
                        it.push(c);
                    }
                    Key::Backspace => {
                        it.pop();
                    },
                    _ => {}
                }
            }
        }).unwrap();
    }
}
