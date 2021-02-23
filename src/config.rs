use yaml_rust::YamlLoader;
use std::convert::AsRef;

pub struct Config {
    pub max_outputs: usize,
}

impl Config {
    pub fn new<S: AsRef<str>>(path: S) -> Config {
        let yaml_data_as_str = std::fs::read_to_string(path.as_ref());

        if let Err(_) = yaml_data_as_str {
            return Config::default();
        }

        let yaml_data = YamlLoader::load_from_str(&yaml_data_as_str.unwrap()[..]);

        if let Err(_) = yaml_data {
            return Config::default();
        }
        
        let yaml_data = &yaml_data.unwrap()[0];

        Config {
            max_outputs: yaml_data["output"]["max_outputs"]
                .clone()
                .into_i64()
                .unwrap() as usize,
        }
    }
}
impl Default for Config {
    fn default() -> Config {
        Config {
            max_outputs: 4
        }
    }
}
