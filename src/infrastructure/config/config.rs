use serde::Deserialize;
use std::fs::File;
use serde_yaml_bw;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub database_url: String,
    pub memgraph_uri: String,
    pub memgraph_user: String,
    pub memgraph_password: String,
}
#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: u64,
    pub idle_timeout: u64,
    pub max_lifetime: u64
}

#[derive(Debug, Deserialize)]
pub struct MemgraphConfig {
    pub max_connections: usize,
    pub fetch_size: usize
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub(crate) db: DatabaseConfig,
    pub(crate) memgraph: MemgraphConfig
}

pub fn load_settings() -> Settings {
    envy::from_env::<Settings>()
        .expect("Invalid environment configuration.")
}

pub fn load_config() -> Config {
    let file = File::open("./config.yaml")
        .expect("Failed to read config file.");
    serde_yaml_bw::from_reader::<File, Config>(file)
        .expect("Failed to parse config file.")
}