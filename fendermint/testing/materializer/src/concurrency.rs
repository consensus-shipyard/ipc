pub struct Config {
    pub parallelism_level: usize
}

impl Config {
    pub fn with_parallelism_level(v: usize) -> Self {
        Self { parallelism_level: v }
    }
}




