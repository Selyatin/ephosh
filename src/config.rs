pub struct Config {
    pub max_outputs: usize,
}

impl Config {
    pub fn new() -> Config {
        Config {
            max_outputs: 4,
        }
    }
}
