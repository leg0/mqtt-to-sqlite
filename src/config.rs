use std::{collections::BTreeMap, fs};
use paho_mqtt::{ QOS_1, QOS_2 };
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

fn default_qos() -> i32 { QOS_1 }

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct LwtSettings
{
    pub topic: String,
    pub message: String,
    #[serde(default = "default_qos")]
    pub qos: i32,
}

impl LwtSettings {
    #[cfg(test)]
    pub fn new(topic: &str, message: &str, qos: i32 ) -> Self {
        Self {
            topic: topic.to_string(),
            message: message.to_string(),
            qos,
        }
    }
}

fn default_client_key() -> String { String::from("/etc/mqtt-to-sqlite/client.key") }
fn default_client_cert() -> String { String::from("/etc/mqtt-to-sqlite/client.crt") }
fn default_ca_cert() -> String { String::from("/etc/mqtt-to-sqlite/ca.crt") }

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct ClientAuth
{
    #[serde(default = "default_client_key")]
    pub client_key: String,
    #[serde(default = "default_client_cert")]
    pub client_cert: String,
    #[serde(default = "default_ca_cert")]
    pub ca_cert: String,
}

impl ClientAuth {
    #[cfg(test)]
    pub fn new(client_key: &str, client_cert: &str, ca_cert: &str) -> Self {
        Self {
            client_key: client_key.to_string(),
            client_cert: client_cert.to_string(),
            ca_cert: ca_cert.to_string(),
        }
    }
}

impl Default for ClientAuth {
    fn default() -> Self {
        Self {
            client_key: default_client_key(),
            client_cert: default_client_cert(),
            ca_cert: default_ca_cert(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Config
{
    pub uri: String, // ws, tcp, ssl
    pub lwt: Option<LwtSettings>,
    pub client_auth: Option<ClientAuth>,
    pub db: String,
    pub metrics: BTreeMap<String,Topic>
}

impl Config {
    pub fn load(file: &str) -> Result<Self, M2SError> {
        let toml_str = fs::read_to_string(file)?;
        Self::load_from_str(&toml_str)
    }

    pub fn load_from_str(toml_str: &str) -> Result<Self, M2SError> {
        Ok(toml::from_str(toml_str)?)
    }

    pub fn get_mqtt_topics(&self) -> Vec<&str> {
        self.metrics
            .values()
            .into_iter()
            .map(| kv: &Topic| { kv.mqtt_topic.as_str() })
            .collect()
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_load_from_str() {
        let toml_str = r#"
            uri = "ws://localhost:1883"
            db = "test.db"
            [metrics]
            "test" = { mqtt_topic = "test", json_path = "test" }

            [metrics.test2]
            mqtt_topic = "test2"
            json_path = "test2"
        "#;
        let config = Config::load_from_str(toml_str).unwrap();
        assert_eq!(config.uri, "ws://localhost:1883");
        assert_eq!(config.db, "test.db");
        assert_eq!(config.metrics.len(), 2);
        assert_eq!(config.metrics.get("test").unwrap().mqtt_topic, "test");
        assert_eq!(config.metrics.get("test").unwrap().json_path, "test");

        assert_eq!(config.metrics.get("test2").unwrap().json_path, "test2");
        assert_eq!(config.metrics.get("test2").unwrap().mqtt_topic, "test2");
    }

    #[test]
    fn test_load_metrics() {
        let toml_str = r#"
            uri = "ws://localhost:1883"
            db = "test.db"
            [metrics]
            "test" = { mqtt_topic = "test", json_path = "test" }
            
            [metrics.test2]
            mqtt_topic = "test2"
            json_path = "test2"
        "#;
        let config = Config::load_from_str(toml_str).unwrap();
        assert_eq!(config.metrics.len(), 2);
        assert_eq!(config.metrics.get("test").unwrap().mqtt_topic, "test");
        assert_eq!(config.metrics.get("test").unwrap().json_path, "test");
        assert_eq!(config.metrics.get("test2").unwrap().json_path, "test2");
        assert_eq!(config.metrics.get("test2").unwrap().mqtt_topic, "test2");
    }

    #[test]
    fn test_load_client_auth_default() {
        let toml_str = r#"
            uri = "ws://localhost:1883"
            db = "test.db"
            [client_auth]
            [metrics]
            "test" = { mqtt_topic = "test", json_path = "test" }
        "#;
        let config = Config::load_from_str(toml_str).unwrap();
        assert_eq!(Some(ClientAuth::default()), config.client_auth);
    }

    #[test]
    fn test_load_clint_auth() {
        let toml_str = r#"
            uri = "ws://localhost:1883"
            db = "test.db"
            [client_auth]
            client_key = "test.key"
            client_cert = "test.crt"
            ca_cert = "test.ca"
            [metrics]
            "test" = { mqtt_topic = "test", json_path = "test" }
        "#;
        let config = Config::load_from_str(toml_str).unwrap();
        assert_eq!(Some(ClientAuth::new("test.key", "test.crt", "test.ca")), config.client_auth);
    }

    #[test]
    fn test_load_lwt_default() {
        let toml_str = r#"
            uri = "ws://localhost:1883"
            db = "test.db"
            [lwt]
            topic = "test"
            message = "test"
            [metrics]
            "test" = { mqtt_topic = "test", json_path = "test" }
        "#;
        let config = Config::load_from_str(toml_str).unwrap();
        assert_eq!(Some(LwtSettings::new("test", "test", default_qos())), config.lwt);
    }

    #[test]
    fn test_load_lwt() {
        let toml_str = r#"
            uri = "ws://localhost:1883"
            db = "test.db"
            [lwt]
            topic = "test"
            message = "test"
            qos = 2
            [metrics]
            "test" = { mqtt_topic = "test", json_path = "test" }
        "#;
        let config = Config::load_from_str(toml_str).unwrap();
        assert_eq!(Some(LwtSettings::new("test", "test", paho_mqtt::QOS_2)), config.lwt);
    }
}