use std::{process, time::Duration};
use crate::config::Config;
use crate::error::M2SError;
use paho_mqtt::{AsyncClient, CreateOptionsBuilder, SslOptions, SslOptionsBuilder, ConnectOptionsBuilder, MQTT_VERSION_DEFAULT, Message, ConnectOptions};

pub fn make_client(config: &Config) -> AsyncClient {
    if let Some(ref ssl) = config.client_auth {
        println!("SSL options:");
        println!("  client key: {}", ssl.client_key);
        println!("  client cert: {}", ssl.client_cert);
        println!("  ca cert: {}", ssl.ca_cert);
    }

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
    if let Some(ref ssl_opts) = config.client_auth {
        let mut builder = SslOptionsBuilder::new();
        builder
            .trust_store(&ssl_opts.ca_cert)?
            .key_store(&ssl_opts.client_cert)?
            .private_key(&ssl_opts.client_key)?;
        Ok(Some(builder.finalize()))
    }
    else {
        Ok(None)
    }
}

pub fn connection_options(config: &Config) -> Result<ConnectOptions, M2SError> {
    
    let mut conn_opts_builder = ConnectOptionsBuilder::new();
    conn_opts_builder
        .keep_alive_interval(Duration::from_secs(30))
        .mqtt_version(MQTT_VERSION_DEFAULT)
        .clean_session(true);

    if let Some(ref lwt) = &config.lwt {
        let msg = Message::new(lwt.topic.to_owned(), lwt.message.to_owned(), lwt.qos);
        conn_opts_builder.will_message(msg);
    }

    if let Some(ssl_opts) = ssl_options(&config)? {
        conn_opts_builder.ssl_options(ssl_opts);
    }

    Ok(conn_opts_builder.finalize())
}
