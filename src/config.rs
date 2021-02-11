use yaml_rust::YamlLoader;
pub struct Config {
    pub max_outputs: usize,
}

impl Config {
    pub fn new(path: &str) -> Config {
        let yaml_data_as_str = std::fs::read_to_string(path);

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
