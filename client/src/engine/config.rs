use std::fs;

use log::{error, info};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_port")]
    pub port: u32,

    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_resolution")]
    pub resolution: [u32; 2],

    #[serde(default = "default_position")]
    pub position: [i32; 2],

    #[serde(default = "default_windowed")]
    pub windowed: bool,

    #[serde(default = "default_save_file")]
    pub save_file: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            port: default_port(),
            host: default_host(),
            resolution: default_resolution(),
            position: default_position(),
            windowed: default_windowed(),
            save_file: default_save_file(),
        }
    }
}

const APP_CONFIG_PATH: &'static str = "./farmisto.json";

impl AppConfig {
    pub fn load() -> Self {
        match fs::read(APP_CONFIG_PATH) {
            Ok(data) => match serde_json::from_slice(&data) {
                Ok(config) => {
                    info!("Uses {APP_CONFIG_PATH}");
                    config
                }
                Err(error) => {
                    error!("Unable to parse config file, {error:?}");
                    AppConfig::default()
                }
            },
            _ => {
                info!("Uses default config, {APP_CONFIG_PATH} not found");
                AppConfig::default()
            }
        }
    }
}

fn default_port() -> u32 {
    8080
}

fn default_host() -> String {
    String::from("127.0.0.1:8080")
}

fn default_resolution() -> [u32; 2] {
    [1920, 1080]
}

fn default_position() -> [i32; 2] {
    [0, 0]
}

fn default_windowed() -> bool {
    true
}

fn default_save_file() -> String {
    String::from("./assets/database.sqlite")
}
