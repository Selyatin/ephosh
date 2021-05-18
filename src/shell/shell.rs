use crate::{
    config::Config,
};
use super::InputMode;
use std::{
    env,
    fs::{File, OpenOptions},
    path::Path,
};
use portable_pty::{
    PtySystem,
    native_pty_system,
    PtySize
};

pub struct Shell {
    pub username: String,
    pub current_dir: String,
    pub active_pane: usize,
    pub error: String,
    pub input: String,
    pub input_mode: InputMode,
    pub config: Config,
    pub history: File,
    pub status_len: usize,
    pub terminal_size: (u16, u16),
    pub pty: Box<dyn PtySystem>
}

impl Default for Shell {
    fn default() -> Self {
        let home_var = match env::var("HOME") {
            Ok(var) => var,
            Err(_) => panic!("Error: Couldn't get HOME var"),
        };

        let config_path = format!("{}/.config/ephosh/ephosh.json", home_var);

        let mut error: String = String::new();

        let config = match Path::new(&config_path).is_file() {
            true => {
                let config = Config::new(config_path);
                match config {
                    Ok(config) => config,
                    Err(err) => {
                        error = String::from("Config: ") + &err.1[..];
                        err.0
                    }
                }
            }
            false => Config::default(),
        };

        let current_dir = env::current_dir().unwrap().to_str().unwrap().to_owned();

        let username = env::var("USER").unwrap().to_owned();

        let history = OpenOptions::new()
            .write(true)
            .read(true)
            .open(&config.history_path);

        let history = if let Err(_) = history {
            File::create(&config.history_path).unwrap()
        } else {
            history.unwrap()
        };
    
        let pty = native_pty_system();
        
        let terminal_size = termion::terminal_size().expect("Couldn't get terminal size");

        Self {
            username,
            current_dir,
            config,
            error,
            input: "".to_owned(),
            input_mode: InputMode::Command,
            active_pane: 0,
            history,
            status_len: 0,
            terminal_size,
            pty
        }
    }
}
