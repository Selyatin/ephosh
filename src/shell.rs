use super::{
    config::Config,
    ui::{input::InputMode, Cursor, Pane},
};
use std::{
    env,
    fs::{File, OpenOptions},
    path::Path,
};
use tui::layout::Rect;

pub struct Shell {
    pub username: String,
    pub current_dir: String,
    pub panes: Vec<Pane>,
    pub active_pane: usize,
    pub error: String,
    pub input: String,
    pub input_mode: InputMode,
    pub cursor: Cursor,
    pub config: Config,
    pub chunks: Vec<Rect>,
    pub history: File,
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

        Self {
            username,
            current_dir,
            config,
            error,
            input: "".to_owned(),
            input_mode: InputMode::Command,
            panes: vec![],
            active_pane: 0,
            cursor: Cursor::new(1, 1),
            chunks: vec![],
            history,
        }
    }
}
