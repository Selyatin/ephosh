use crossterm::event::{self, KeyEvent};
use std::{sync::mpsc, thread, time::Duration};

pub enum Event<I> {
    Input(I),
    Tick,
}

pub struct Events {
    rx: mpsc::Receiver<Event<KeyEvent>>,
    _tx: mpsc::Sender<Event<KeyEvent>>,
}

impl Events {
    pub fn new(tick_rate: u64) -> Events {
        let (tx, rx) = mpsc::channel();

        let event_tx = tx.clone();
        let tick_rate = Duration::from_millis(tick_rate);

        thread::spawn(move || {
            loop {
                if event::poll(tick_rate).unwrap() {
                    if let event::Event::Key(key) = event::read().unwrap() {
                        event_tx.send(Event::Input(key)).unwrap();
                    }
                }
                event_tx.send(Event::Tick).unwrap();
            }
        });

        Events { rx, _tx: tx }
    }

    pub fn next(&self) -> Result<Event<KeyEvent>, mpsc::RecvError> {
        self.rx.recv()
    }
}
