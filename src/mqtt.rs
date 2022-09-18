use std::{process, time::Duration};
use crate::config::Config;
use crate::error::M2SError;
use paho_mqtt::{AsyncClient, CreateOptionsBuilder, SslOptions, SslOptionsBuilder, ConnectOptionsBuilder, MQTT_VERSION_DEFAULT, Message, QOS_1, ConnectOptions};

pub fn make_client(config: &Config) -> AsyncClient {
    if let Some(ref ca_cert) = config.ca_cert { println!("CA cert: {}", ca_cert); }
    if let Some(ref client_cert) = config.client_cert { println!("client cert: {}", client_cert); }
    if let Some(ref client_key) = config.client_key { println!("client cert: {}", client_key); }

     let create_opts = CreateOptionsBuilder::new()
         .server_uri(&config.uri)
         .client_id("")
         .finalize();

    AsyncClient::new(create_opts).unwrap_or_else(|e| {
        println!("Error creating the client: {:?}", e);
        process::exit(1);
    })
}

pub fn ssl_options(config: &Config) -> Result<Option<SslOptions>, M2SError> {
    match (&config.ca_cert, &config.client_key, &config.client_cert) {
        (Some(ref ca), Some(ref key), Some(ref crt)) => {
            let mut builder = SslOptionsBuilder::new();
            builder
                .trust_store(ca)?
                .key_store(crt)?
                .private_key(key)?;
            Ok(Some(builder.finalize()))    
        }
        _ => {
            Ok(None)
        }
    }
}

pub fn connection_options(config: &Config) -> Result<ConnectOptions, M2SError> {
    let lwt = Message::new("test", "Async subscriber lost connection", QOS_1);
    let mut conn_opts_builder = ConnectOptionsBuilder::new();
    conn_opts_builder
        .keep_alive_interval(Duration::from_secs(30))
        .mqtt_version(MQTT_VERSION_DEFAULT)
        .clean_session(true)
        .will_message(lwt);
    if let Some(ssl_opts) = ssl_options(&config)? {
        conn_opts_builder.ssl_options(ssl_opts);
    }
    Ok(conn_opts_builder.finalize())
}
