use common::constants::{BUS_ADDRESS, OBJECT_PATH, WELL_KNOWN_NAME};
use common::test1::{Test1, Test1Signals};
use std::time::{SystemTime, UNIX_EPOCH};
use zbus::connection::Builder;
use zbus::{Address, Connection, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let test1 = Test1 {
        state: "idle".into(),
    };

    let conn = build(test1).await?;
    run_heartbeat_loop(&conn).await
}

async fn build(test1: Test1) -> Result<Connection> {
    let address_str =
        std::env::var("DBUS_SESSION_BUS_ADDRESS").unwrap_or_else(|_| BUS_ADDRESS.to_string());

    let address = Address::try_from(address_str.as_str())?;
    println!("Connecting to D-Bus at address: {}", address);

    let connection = Builder::address(address)?
        .name(WELL_KNOWN_NAME)?
        .serve_at(OBJECT_PATH, test1)?
        .build()
        .await?;

    Ok(connection)
}

async fn run_heartbeat_loop(conn: &zbus::Connection) -> Result<()> {
    loop {
        let timestamp = current_unix_timestamp();
        emit_heartbeat(conn, timestamp).await;
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

async fn emit_heartbeat(conn: &zbus::Connection, timestamp: u64) {
    match conn.object_server().interface(OBJECT_PATH).await {
        Ok(interface) => {
            if let Err(e) = interface.heartbeat(timestamp).await {
                eprintln!("Failed to emit heartbeat signal: {}", e);
            } else {
                println!("Heartbeat timestamp: {}", timestamp);
            }
        }
        Err(e) => {
            eprintln!("Failed to get object server interface: {}", e);
        }
    }
}
