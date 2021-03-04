use std::io;
use std::sync::mpsc;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;
use termion::event::{Event, Key};
use termion::input::TermRead;

#[derive(PartialEq)]
pub enum InputMode {
    Command,
    Interact,
}

pub enum TerminalEvent<I> {
    Input(I),
    Tick,
}

/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
pub struct Events {
    rx: mpsc::Receiver<TerminalEvent<Event>>,
    //input_handle: thread::JoinHandle<()>,
    //tick_handle: thread::JoinHandle<()>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub exit_key: Key,
    pub tick_rate: Duration,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            exit_key: Key::Char('q'),
            tick_rate: Duration::from_millis(150),
        }
    }
}

impl Events {
    pub fn new() -> Events {
        Events::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Events {
        let (tx, rx) = mpsc::channel();

        let ignore_exit_key = Arc::new(AtomicBool::new(false));

        let _input_handle = {
            let tx = tx.clone();

            let ignore_exit_key = ignore_exit_key.clone();

            thread::spawn(move || {
                let stdin = io::stdin();

                for evt in stdin.events() {
                    if let Ok(event) = evt {
                        if let Err(err) = tx.send(TerminalEvent::Input(event.clone())) {
                            eprintln!("{}", err);

                            return;
                        }

                        if !ignore_exit_key.load(Ordering::Relaxed)
                            && event == Event::Key(config.exit_key)
                        {
                            return;
                        }
                    }
                }
            })
        };
        let _tick_handle = {
            thread::spawn(move || loop {
                if tx.send(TerminalEvent::Tick).is_err() {
                    break;
                }
                thread::sleep(config.tick_rate);
            })
        };
        Events {
            rx,
            //    input_handle,
            //    tick_handle,
        }
    }

    pub fn next(&self) -> Result<TerminalEvent<Event>, mpsc::RecvError> {
        self.rx.recv()
    }
}
