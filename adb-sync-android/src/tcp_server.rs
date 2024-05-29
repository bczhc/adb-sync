use adb_sync::stream::android::handle_connection;
use clap::Parser;
use std::net::{SocketAddr, TcpListener};

#[derive(clap::Parser, Debug)]
struct Args {
    port: u16,
}

pub fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let listener = TcpListener::bind(
        format!("0.0.0.0:{}", args.port)
            .parse::<SocketAddr>()
            .unwrap(),
    )?;
    loop {
        let (mut stream, from) = listener.accept()?;
        println!("Connected from: {}", from);

        let result = handle_connection(&mut stream);
        println!("{:?}", result);
    }
}
