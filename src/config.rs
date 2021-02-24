use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub max_outputs: usize,
    pub history_path: String,
    pub history_size: usize,
}

impl Config {
    pub fn new<S: AsRef<str>>(path: S) -> Result<Config, (Config, String)> {
        let json_data_as_str = std::fs::read_to_string(path.as_ref());

        if let Err(err) = json_data_as_str {
            return Err((Config::default(), err.to_string()));
        }

        let json_data = serde_json::from_str(&json_data_as_str.unwrap()[..]);

        if let Err(err) = json_data {
            return Err((Config::default(), err.to_string()));
        }
        
        let json_data: Config = json_data.unwrap();

        let history_path = json_data.history_path.to_string();

        let history_path = match history_path.is_empty() {
            true => format!("{}/.ephosh_history", std::env::var("HOME").unwrap()),
            false => history_path,
        };

        let history_size = match json_data.history_size.to_string().is_empty() {
            true => 1000,
            false => json_data.history_size,
        };

        Ok(Config {
            max_outputs: json_data.max_outputs,
            history_path,
            history_size,
        })
    }
}
impl Default for Config {
    fn default() -> Config {
        Config {
            max_outputs: 4,
            history_path: format!("{}/.ephosh_history", std::env::var("HOME").unwrap()),
            history_size: 1000,
        }
    }
}
