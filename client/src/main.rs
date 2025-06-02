use common::constants::{BUS_ADDRESS, INTERFACE, OBJECT_PATH, WELL_KNOWN_NAME};
use common::test1::Test1Proxy;
use futures_util::StreamExt;
use std::time::Duration;
use zbus::connection::Builder;
use zbus::fdo::DBusProxy;
use zbus::proxy::PropertyChanged;
use zbus::{Address, Connection, Result};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let conn = connect_client().await?;
    let proxy = build(&conn).await?;
    listen_to_signals(proxy).await?;

    anyhow::bail!("Program ended");
}

async fn connect_client() -> Result<Connection> {
    let address_str =
        std::env::var("DBUS_SESSION_BUS_ADDRESS").unwrap_or_else(|_| BUS_ADDRESS.to_string());

    let address = Address::try_from(address_str.as_str())?;
    println!("Connecting to D-Bus at address: {}", address);
    let connection = Builder::address(address)?.build().await?;
    wait_for_service(&connection).await?;

    Ok(connection)
}

async fn build(connection: &Connection) -> Result<Test1Proxy<'_>> {
    Test1Proxy::builder(connection)
        .interface(INTERFACE)?
        .path(OBJECT_PATH)?
        .destination(WELL_KNOWN_NAME)?
        .build()
        .await
}

async fn wait_for_service(connection: &Connection) -> Result<()> {
    let dbus_proxy = DBusProxy::new(connection).await?;
    println!("Waiting for 'org.zbus.Test' to appear on the bus...");

    loop {
        let owners = dbus_proxy.list_names().await?;
        if owners.iter().any(|name| *name == "org.zbus.Test") {
            println!("Found 'org.zbus.Test' on the bus!");
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}

async fn listen_to_signals(proxy: Test1Proxy<'_>) -> anyhow::Result<()> {
    let mut state_stream = proxy.receive_state_changed().await;
    let mut heartbeat_stream = match proxy.receive_heartbeat().await {
        Ok(stream) => stream,
        Err(e) => {
            anyhow::bail!("Failed to subscribe to heartbeat: {}", e);
        }
    };

    loop {
        tokio::select! {
            Some(state_signal) = state_stream.next() => {
                if let Err(e) = handle_state_change(state_signal).await {
                    eprintln!("Error handling state change: {}", e);
                }
            }
            Some(heartbeat_signal) = heartbeat_stream.next() => {
                if let Err(e) = handle_heartbeat(heartbeat_signal.message(), &proxy).await {
                    eprintln!("Error handling heartbeat: {}", e);
                }
            }
            else => break,
        }
    }

    Ok(())
}

async fn handle_state_change(signal: PropertyChanged<'_, String>) -> Result<()> {
    if let Ok(state) = signal.get().await {
        println!("State changed: {:?}", state);
    }
    Ok(())
}

async fn handle_heartbeat(msg: &zbus::Message, proxy: &Test1Proxy<'_>) -> Result<()> {
    match deserialize_heartbeat(msg) {
        Ok(timestamp) => {
            println!("Heartbeat timestamp: {}", timestamp);
            toggle_proxy_state(proxy).await?;
        }
        Err(e) => {
            eprintln!("Failed to decode heartbeat: {}", e);
        }
    }
    Ok(())
}

fn deserialize_heartbeat(msg: &zbus::Message) -> std::result::Result<u64, zbus::Error> {
    msg.body().deserialize::<u64>()
}

async fn toggle_proxy_state(proxy: &Test1Proxy<'_>) -> Result<()> {
    match proxy.state().await?.as_str() {
        "active" => deactivate_proxy(proxy).await,
        _ => activate_proxy(proxy).await,
    }
}

async fn deactivate_proxy(proxy: &Test1Proxy<'_>) -> Result<()> {
    println!("Proxy is active, deactivating...");
    if let Err(e) = proxy.deactivate().await {
        eprintln!("Failed to deactivate: {}", e);
    }
    Ok(())
}

async fn activate_proxy(proxy: &Test1Proxy<'_>) -> Result<()> {
    println!("Proxy is inactive, activating...");
    if let Err(e) = proxy.activate().await {
        eprintln!("Failed to activate: {}", e);
    }
    Ok(())
}
