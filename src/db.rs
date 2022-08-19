use rusqlite::Connection;
use crate::config::Config;

pub fn initialize_database(conn: &Connection, config: &Config) -> Result<(), Box<dyn std::error::Error>>
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
