use config::Metric;
use futures::{executor::block_on, stream::StreamExt};
use paho_mqtt::{QOS_2, Message};
use serde_json::Value;
use std::collections::BTreeMap;
use std::env;

use jq_rs as jq;
use std::time::Duration;
use rusqlite::{Connection, Result, params};

mod config;
mod db;
mod mqtt;
mod error;
use crate::{config::Config, error::M2SError};

const DEFAULT_CONFIG_FILE_PATH: &str = "/etc/mqtt-to-sqlite/mqtt-to-sqlite.toml";
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn handle_message(
    msg: &Message,
    topic_to_metrics: &BTreeMap<&str, Vec<&Metric>>,
    conn: &Connection) -> Result<(), M2SError>
{
    let timestamp = chrono::Utc::now().timestamp_millis();
    println!("{msg}");
    let topic = msg.topic();
    let payload = msg.payload_str();

    if !topic_to_metrics.contains_key(topic) {
        println!("Warning: Config contains wildcard. Not supported yet");
    }
    else if let Some(metrics) = topic_to_metrics.get(topic) {
        for &metric in metrics {
            let program = format!("({})?", metric.json_path);
            // TODO: optimize: pre-compile the jq query, and re-use
            let result = jq::run(&program, &payload)?;
            let sqlvalue = match serde_json::from_str::<Value>(&result)? {
                Value::Bool(_)   => true,
                Value::Number(_) => true,
                Value::String(_) => true,
                Value::Null =>
                {
                    println!("Query '{}' failed to find anything from payload '{payload}'", program);
                    false
                },
                val => {
                    println!("Unsupported value: {val}");
                    false
                },
            };
            if sqlvalue {
                let sql = format!("insert into {} (t, value) values(?,?)", metric.metric_name);
                conn.execute(&sql, params![timestamp, result.to_string()])?;
            }
        }
    }
    Ok(())
}

fn main() -> Result<(), M2SError>
{
    let mut config_file_name = DEFAULT_CONFIG_FILE_PATH.to_owned();
    let args = &mut env::args();
    args.next(); // drop the first arg, as that's the executable.
    match args.next().as_ref().map(|s| &s[..]) {
        Some("--version") => {
            println!("mqtt-to-sqlite {VERSION}");
            return Ok(());
        }
        Some("--config") => {
            match args.next() {
                Some(cfg_path) => {
                    config_file_name = cfg_path.to_owned();
                }
                _ => {
                    panic!("Expected file path");
                }
            }
        }
        _ => {}
    }

    // Configure
    let config = Config::load(&config_file_name)
        .expect(&format!("Failed to load config from file {config_file_name}"));

    // Make map from topic -> Vec<&Metric>
    let metrics = &config.metrics;
    let mut topic_to_metrics: BTreeMap<&str, Vec<&Metric>> = BTreeMap::new();
    for metric in &*metrics {
        let topic = metric.mqtt_topic.as_str();
        if let Some(metrics) = topic_to_metrics.get_mut(topic) {
            metrics.push(metric);
        }
        else {
            topic_to_metrics.insert(topic, vec![metric]);
        }
    }
    let topic_to_metrics = &topic_to_metrics;

    // Database
    let conn = Connection::open(&config.db)?;
    db::initialize_database(&conn, &config)?;

    // MQTT
    let mut mqtt_client = mqtt::make_client(&config);

    if let Err(err) = block_on(async {
        println!("Connecting...");
        // Get message stream before connecting.
        let mut strm = mqtt_client.get_stream(25);
 
        // Define the set of options for the connection

        let conn_opts = mqtt::connection_options(&config)?;
        mqtt_client.set_disconnected_callback(|_a,_b,c| {
            println!("cb: client disconnected reason={:?}", c);
        });
        mqtt_client.set_connection_lost_callback(|_client| {
            println!("cb: Connection lost");
        });
        mqtt_client.set_connected_callback(|_a| {
            println!("cb: connected");
        });

        mqtt_client.connect(conn_opts).await?;

        let topics: Vec<&str> = config.get_mqtt_topics().into_iter().collect();
        let qos = vec![QOS_2; topics.len()]; //TODO: make configurable
        println!("Subscribing to topics: {:?}", topics);
        let _subs = mqtt_client.subscribe_many(&topics[..], &qos[..]).await?;
 
         // Just loop on incoming messages.
         println!("Waiting for messages...");
 
         // Note that we're not providing a way to cleanly shut down and
         // disconnect. Therefore, when you kill this app (with a ^C or
         // whatever) the server will get an unexpected drop and then
         // should emit the LWT message.
 
        while let Some(msg_opt) = strm.next().await {
            if let Some(ref msg) = msg_opt {
                handle_message(&msg, &topic_to_metrics, &conn)?;
            }
            else {
                // A "None" means we were disconnected. Try to reconnect...
                println!("Lost connection. Attempting reconnect.");
                while let Err(err) = mqtt_client.reconnect().await {
                    println!("Error reconnecting: {}", err);
                    // For tokio use: tokio::time::delay_for()
                    async_std::task::sleep(Duration::from_millis(1000)).await;
                }
                println!("Reconnected");
            }
        }

        // Explicit return type for the async block
        Ok::<(), M2SError>(())
    }) {
        eprintln!("{:?}", err);
    }    
    Ok(())
}
