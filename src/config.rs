use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub max_outputs: usize,
    pub history_path: String,
    pub history_size: usize,
}

impl Config {
    pub fn new<S: AsRef<str>>(path: S) -> Result<Config, (Config, String)> {

        let json_data_as_str = match std::fs::read_to_string(path.as_ref()){
            Ok(data) => data,
            Err(err) => return Err((Config::default(), err.to_string()))
        };

        let mut config: Config = match serde_json::from_str(&json_data_as_str){
            Ok(config) => config,
            Err(err) => return Err((Config::default(), err.to_string()))
        };

        
        if config.history_path.is_empty() {
            config.history_path = format!("{}/.ephosh_history", std::env::var("HOME").unwrap());
        }
        
        Ok(config)
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
