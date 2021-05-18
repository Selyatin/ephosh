use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub enum Key {
    Char(char),
    Ctrl(char),
    Esc,
    Unknown,
}

impl From<KeyEvent> for Key {
    fn from(data: KeyEvent) -> Self {
        match data {
            KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::CONTROL,
            } => Key::Ctrl(c),

            KeyEvent {
                code: KeyCode::Char(c),
                ..
            } => Key::Char(c),
            
            KeyEvent {
                code: KeyCode::Esc,
                ..
            } => Key::Esc,

            _ => Key::Unknown,
        }
    }
}
