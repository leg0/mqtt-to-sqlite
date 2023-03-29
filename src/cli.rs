use clap::Parser;


pub const DEFAULT_CONFIG_FILE_PATH: &str = "/etc/mqtt-to-sqlite/mqtt-to-sqlite.toml";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
//#[derive(Debug)]
#[command(name="mqtt-to-sqlite", version=VERSION, about="MQTT to SQLite bridge")]
pub(crate) struct Cli {
    #[arg(long = "config", short = 'c', default_value_t = DEFAULT_CONFIG_FILE_PATH.to_owned())]
    pub(crate) config_file_name: String,
}
