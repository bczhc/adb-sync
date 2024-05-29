use std::net::{SocketAddr, TcpListener};

use adb_sync::stream::android::handle_connection;
use adb_sync::ADB_SYNC_PORT;

pub fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind(
        format!("0.0.0.0:{}", ADB_SYNC_PORT)
            .parse::<SocketAddr>()
            .unwrap(),
    )?;
    let (mut stream, from) = listener.accept()?;
    println!("Connected from: {}", from);

    handle_connection(&mut stream)?;
    Ok(())
}
