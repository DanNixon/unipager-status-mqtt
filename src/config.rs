use anyhow::Result;
use serde::Deserialize;
use std::{fs, path::Path};
use url::Url;

#[derive(Deserialize)]
pub(crate) struct Config {
    pub unipager: UnipagerConfig,
    pub mqtt: MqttConfig,
    pub topics: TopicsConfig,
}

impl Config {
    pub fn from_file(filename: &Path) -> Result<Self> {
        Ok(toml::from_str(&fs::read_to_string(filename)?)?)
    }
}

#[derive(Deserialize)]
pub(crate) struct UnipagerConfig {
    pub api: Url,
}

#[derive(Deserialize)]
pub(crate) struct MqttConfig {
    pub broker: Url,
    pub client_id: String,
    pub username: String,
    pub password: String,
}

#[derive(Clone, Deserialize)]
pub(crate) struct TopicsConfig {
    pub availability: String,
    pub timeslot: String,
    pub queue_length: String,
    pub transmitting: String,
    pub new_message: String,
}
