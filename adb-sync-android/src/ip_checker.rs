use adb_sync::{any_ipv4_socket, IP_CHECKER_PORT};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::process;
use std::thread::{sleep, spawn};
use std::time::Duration;

pub fn main() -> anyhow::Result<()> {
    spawn(|| {
        sleep(Duration::from_secs(2));
        process::exit(0);
    });

    // a simple echo server, to test the connectivity
    let socket_addr = any_ipv4_socket(IP_CHECKER_PORT);
    let listener = TcpListener::bind(socket_addr).unwrap();
    println!("Listening on {}", socket_addr);
    let (mut stream, addr) = listener.accept()?;
    println!("Connected from: {}", addr);
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    drop(reader);
    writeln!(&mut stream, "{}", line)?;
    Ok(())
}
