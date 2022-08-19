#[derive(Debug)]
pub enum M2SError
{
    MqttError(paho_mqtt::Error),
    SqliteError(rusqlite::Error),
    JqError(jq_rs::Error),
    JsonError(serde_json::Error),
    //OtherError
}

impl From<paho_mqtt::Error> for M2SError {
    fn from(e: paho_mqtt::Error) -> Self {
        M2SError::MqttError(e)
    }
}

impl From<rusqlite::Error> for M2SError {
    fn from(e: rusqlite::Error) -> Self {
        M2SError::SqliteError(e)
    }
}

impl From<jq_rs::Error> for M2SError {
    fn from(e: jq_rs::Error) -> Self {
        M2SError::JqError(e)
    }
}

impl From<serde_json::Error> for M2SError {
    fn from(e: serde_json::Error) -> Self {
        M2SError::JsonError(e)
    }
}
