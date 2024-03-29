use futures::{executor::block_on, stream::StreamExt};
use paho_mqtt::{QOS_2, Message};
use serde_json::Value;
use std::collections::BTreeMap;

use jq_rs as jq;
use std::time::Duration;
use rusqlite::{Connection, Result, params};
use clap::Parser;

mod config;
mod cli;
mod db;
mod mqtt;
mod error;
use crate::{config::Config, error::M2SError};
use crate::cli::Cli;

fn handle_message(
    msg: &Message,
    mq_to_jq_and_metric: &BTreeMap<&str, (&str,&str)>,
    conn: &Connection) -> Result<(), M2SError>
{
    let timestamp = chrono::Utc::now().timestamp_millis();
    println!("{msg}");
    let topic = msg.topic();
    let payload = msg.payload_str();

    if !mq_to_jq_and_metric.contains_key(topic) {
        println!("Config contains wildcard. Not supported yet");
    }
    else if let Some((ref query, ref metric_name)) = mq_to_jq_and_metric.get(topic) {
        let query2 = format!("({})?", query);
        // TODO: optimize: pre-compile the jq query, and re-use
        let result = jq::run(&query2, &payload)?;
        let sqlvalue = match serde_json::from_str::<Value>(&result)? {
            Value::Bool(_)   => true,
            Value::Number(_) => true,
            Value::String(_) => true,
            Value::Null =>
            {
                println!("Query '{query2}' failed to find anything from payload '{payload}'");
                false
            },
            val => {
                println!("Unsupported value: {val}");
                false
            },
        };
        if sqlvalue {
            let sql = format!("insert into {metric_name} (t, value) values(?,?)");
            conn.execute(&sql, params![timestamp, result.to_string()])?;
        }
    }
    Ok(())
}

fn main() -> Result<(), M2SError>
{
    let args = Cli::parse();

    // Configure
    let config = Config::load(&args.config_file_name)
        .expect(&format!("Failed to load config from file {}", args.config_file_name));

    // Make map from topic -> (jq query, metric name)
    let metrics = &config.metrics;
    let mut mq_to_metric_and_jq = BTreeMap::new();
    for (k,v) in &*metrics {
        let v2 = (v.json_path.as_str(), k.as_str());
        mq_to_metric_and_jq.insert(v.mqtt_topic.as_str(), v2);
    }

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

        mqtt_client.set_disconnected_callback(|_a,_b,c| {
            println!("cb: client disconnected reason={:?}", c);
        });
        mqtt_client.set_connection_lost_callback(|_client| {
            println!("cb: Connection lost");
        });
        mqtt_client.set_connected_callback(|_a| {
            println!("cb: connected");
        });
        
        let conn_opts = mqtt::connection_options(&config)?;
        mqtt_client.connect(conn_opts).await?;

        let topics = config.get_mqtt_topics();
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
                handle_message(&msg, &mq_to_metric_and_jq, &conn)?;
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
