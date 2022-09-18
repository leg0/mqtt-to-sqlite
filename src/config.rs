use std::{collections::{HashSet, BTreeMap}, fs};
use serde_derive::{Serialize,Deserialize};

use crate::error::M2SError;

#[derive(Deserialize, Serialize, Debug)]
struct MetricModel
{
    pub mqtt_topic: String,
    pub json_path: String,
    pub unit: Option<String>,
    pub description: Option<String>,
}

pub struct Metric {
    pub metric_name: String,
    pub mqtt_topic: String,
    pub json_path: String,
    pub unit: Option<String>,
    pub description: Option<String>,
}

impl Metric {
    fn new(metric_name: String, model: MetricModel) -> Self {
        Self {
            metric_name: metric_name,
            mqtt_topic: model.mqtt_topic,
            json_path: model.json_path,
            unit: model.unit,
            description: model.description
        }
    }
}

#[derive(Deserialize, Serialize)]
struct ConfigModel
{
    pub uri: String, // scheme://host:port

    pub ca_cert: Option<String>,
    pub client_key: Option<String>,
    pub client_cert: Option<String>,

    pub db: String,

    pub metrics: BTreeMap<String, MetricModel> // metric_name -> Metric
}

pub struct Config {
    pub uri: String, // scheme://host:port

    pub ca_cert: Option<String>,
    pub client_key: Option<String>,
    pub client_cert: Option<String>,

    pub db: String,

    pub metrics: Vec<Metric> // metric_name -> Metric
}

impl Config {
    pub fn load(file: &str) -> Result<Self, M2SError> {
        let toml_str = fs::read_to_string(file)?;
        let model: ConfigModel = toml::from_str(&toml_str)?;
        
        let mut metrics = vec![];
        for m in model.metrics {
            metrics.push(Metric::new(m.0, m.1));
        }
        let res = Self {
            uri: model.uri,
            ca_cert: model.ca_cert,
            client_key: model.client_key,
            client_cert: model.client_cert,
            db: model.db,
            metrics: metrics,
        };
        Ok(res)
    }

    pub fn get_mqtt_topics(&self) -> HashSet<&str> {
        self.metrics
            .iter()
            .map(| metric: &Metric| { metric.mqtt_topic.as_str() })
            .collect()
    }
}