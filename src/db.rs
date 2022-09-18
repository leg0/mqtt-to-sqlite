use rusqlite::Connection;
use crate::{config::Config, error::M2SError};

pub fn initialize_database(conn: &Connection, config: &Config) -> Result<(), M2SError>
{
    conn.execute("create table if not exists metadata(metric primary key, unit, description)", [])?;

    // for (metric_name, metric) in &config.metrics {    
    //     println!("=== {}", metric_name);
    //     let m = conn.changes() as usize;
    //     let sql = format!("create table if not exists {metric_name} (t integer primary key asc, value)");
    //     let n = conn.execute(&sql, [])?;
    //     println!("{} -> n={}", sql, n);
    //     let unit = if let Some(ref unit) = metric.unit { unit } else { "" };
    //     let desc = if let Some(ref desc) = metric.description { desc} else { "" };
    //     if n > m {
    //         println!("Table {:?} created, adding metadata", metric);
    //         conn.execute("insert into metadata(metric, unit, description) values(?, ?, ?)", 
    //             &[metric_name, unit, desc ])?;
    //     }
    //     else {
    //         println!("Table {metric_name} already exists");
    //         conn.execute("update metadata set unit=?, description=? where metric=?", 
    //             &[unit, &desc, metric_name])?;
    //     }
    // }
    Ok(())
}
