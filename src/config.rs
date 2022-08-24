use std::{collections::BTreeMap, fs};
use serde_derive::{Serialize,Deserialize};

use crate::error::M2SError;

#[derive(Deserialize, Serialize)]
pub struct Topic
{
    pub mqtt_topic: String,
    pub json_path: String,
    pub unit: Option<String>,
    pub description: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Config
{
    pub uri: String, // ws, tcp, ssl

    pub ca_cert: Option<String>,
    pub client_key: Option<String>,
    pub client_cert: Option<String>,

    pub db: String,

    pub metrics: BTreeMap<String,Topic>
}

impl Config {
    pub(crate) fn load(file: &str) -> Result<Self, M2SError> {
        let toml_str = fs::read_to_string(file)?;
        Ok(toml::from_str(&toml_str)?)
    }

    pub fn get_mqtt_topics(&self) -> Vec<&str> {
        self.metrics
            .values()
            .into_iter()
            .map(| kv: &Topic| { kv.mqtt_topic.as_str() })
            .collect()
    }
}