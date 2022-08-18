use futures::{executor::block_on, stream::StreamExt};
use mqtt::SslOptionsBuilder;
use serde_derive::{Serialize,Deserialize};
use std::{fs, error::Error, collections::BTreeMap};
use paho_mqtt as mqtt;
use serde_json as json;
use std::{process, time::Duration};
use std::option::Option;
use rusqlite::{Connection, Result};

#[derive(Deserialize, Serialize)]
struct Topic
{
    mqtt_topic: String,
    json_path: String,
    unit: Option<String>
}

#[derive(Deserialize, Serialize)]
struct Config
{
    uri: String, // ws, tcp, ssl

    ca_cert: Option<String>,
    client_key: Option<String>,
    client_cert: Option<String>,

    db: String,

    metrics: BTreeMap<String,Topic>
}

const CONFIG_FILE_PATH: &str = "mqtt-to-sqlite.toml";

fn connect_mqtt(config: &Config) -> mqtt::AsyncClient {
    if let Some(ref ca_cert) = config.ca_cert { println!("CA cert: {}", ca_cert); }
    if let Some(ref client_cert) = config.client_cert { println!("client cert: {}", client_cert); }
    if let Some(ref client_key) = config.client_key { println!("client cert: {}", client_key); }

     let create_opts = mqtt::CreateOptionsBuilder::new()
         .server_uri(&config.uri)
         .client_id("mqtt-to-sqlite")
         .finalize();

    mqtt::AsyncClient::new(create_opts).unwrap_or_else(|e| {
        println!("Error creating the client: {:?}", e);
        process::exit(1);
    })
}

fn initialize_database(conn: &Connection, config: &Config) -> Result<(), Box<dyn std::error::Error>>
{
    conn.execute("create table if not exists metadata(metric primary key, unit, description)", [])?;

    for (ref metric, ref x) in &config.metrics {    
        println!("=== {}", metric);
        let m = conn.changes() as usize;
        let sql = format!("create table if not exists {name} (t integer primary key asc, value)", name = metric);
        let n = conn.execute(&sql, [])?;
        println!("{} -> n={}", sql, n);
        let unit = if let Some(ref unit) = x.unit { unit } else { "" };
        let desc = format!("{}, {}", x.mqtt_topic, x.json_path);
        if n > m {
            println!("Table {} created, adding metadata", metric);
            conn.execute("insert into metadata(metric, unit, description) values(?, ?, ?)", 
                &[metric, unit, &desc ])?;
        }
        else {
            println!("Table {} already exists", metric);
            conn.execute("update metadata set unit=?, description=? where metric=?", 
                &[unit, &desc, metric])?;
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>>
{
    // Configure
    let toml_str = fs::read_to_string(CONFIG_FILE_PATH)?;
    let config: Config = toml::from_str(&toml_str)?;

    // Database
    let conn = Connection::open(&config.db)?;
    initialize_database(&conn, &config)?;

    // MQTT
    let mut mqtt_client = connect_mqtt(&config);

     if let Err(err) = block_on(async {
         // Get message stream before connecting.
         let mut strm = mqtt_client.get_stream(25);
 
         // Define the set of options for the connection
         let lwt = mqtt::Message::new("test", "Async subscriber lost connection", mqtt::QOS_1);
 
         let ssl_opts = SslOptionsBuilder::new()
            .trust_store("/home/lego/.step/certs/root_ca.crt")?
            .key_store("/home/lego/mqtt-to-sqlite/client.crt")?
            .private_key("/home/lego/mqtt-to-sqlite/client.key")?
            .finalize();
         let conn_opts = mqtt::ConnectOptionsBuilder::new()
             .keep_alive_interval(Duration::from_secs(30))
             .mqtt_version(mqtt::MQTT_VERSION_3_1_1)
             .clean_session(false)
             .will_message(lwt)
             .ssl_options(ssl_opts)
             .finalize();
 
         // Make the connection to the broker
         println!("Connecting to the MQTT server...");
         mqtt_client.connect(conn_opts).await?;
 
        let topics: Vec<&str> = config.metrics
            .values()
            .into_iter()
            .map(| kv:&Topic| { kv.mqtt_topic.as_str() })
            .collect();
        let qos = vec![1; topics.len()];
        println!("Subscribing to topics: {:?}", topics);
        mqtt_client.subscribe_many(&topics[..], &qos[..]).await?;
 
         // Just loop on incoming messages.
         println!("Waiting for messages...");
 
         // Note that we're not providing a way to cleanly shut down and
         // disconnect. Therefore, when you kill this app (with a ^C or
         // whatever) the server will get an unexpected drop and then
         // should emit the LWT message.
 
         while let Some(msg_opt) = strm.next().await {
             if let Some(ref msg) = msg_opt {
                 println!("{}", msg);
                 let payload = msg.payload_str().to_string();
                 if  let Ok(v) = serde_json::from_str::<json::Value>(&payload) {
                    // TODO use jq to query the value from msg
                 }
            }
             else {
                 // A "None" means we were disconnected. Try to reconnect...
                 println!("Lost connection. Attempting reconnect.");
                 while let Err(err) = mqtt_client.reconnect().await {
                     println!("Error reconnecting: {}", err);
                     // For tokio use: tokio::time::delay_for()
                     async_std::task::sleep(Duration::from_millis(1000)).await;
                 }
             }
         }
 
         // Explicit return type for the async block
         Ok::<(), mqtt::Error>(())
     }) {
         eprintln!("{}", err);
     }    
    Ok(())
}
