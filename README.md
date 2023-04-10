# mqtt-to-sqlite

Simple utility that subscribes to MQTT topics, and stores values in SQLite3 database along with timestamps. This is to enable the use of 
the data as data source for Grafana for example.

The utility is configured with a toml file that looks like this:
```toml
uri = "ssl://mosquitto.lan:8883"

[client_auth]
ca_cert =  "root_ca.crt"
# The client cert issued by the CA that the ca_cert belongs to.
client_cert = "client.crt"
client_key = "client.key"

# Path to the sqlite3 database file
db = "m2s.db"

# For now, it is expected that the payload os a message
# is in JSON format.

# When a message for topic a/c/b is received, a jq program given in json_path
# is run, and the result is written to the table 'metric_name'.
[metrics.metric_name]
mqtt_topic = "a/c/b"
json_path = ".q.w.e"

# This goes to metadata table
unit = "C"

# This goes to table 'another_metric'.
[metrics.another_metric]
mqtt_topic = "c/d/e"
json_path = ".z.x.c"
unit = "m"
```

## Create .deb package

```
cargo deb --target armv7-unknown-linux-gnueabihf
cargo deb
```
