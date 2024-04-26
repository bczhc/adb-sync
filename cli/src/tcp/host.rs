use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;

use clap::Parser;

use adb_sync::stream::host::start;
use adb_sync::stream::protocol::SendConfig;

#[derive(clap::Parser, Debug)]
struct Args {
    address: String,
    android_dir: PathBuf,
    dest_dir: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut stream = TcpStream::connect(args.address.parse::<SocketAddr>()?)?;

    start(
        &mut stream,
        SendConfig {
            path: args.android_dir,
        },
        &args.dest_dir,
    )?;

    Ok(())
}
