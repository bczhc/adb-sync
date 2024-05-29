use adb_sync::{any_ipv4_socket, IP_CHECKER_PORT};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::process;
use std::thread::{sleep, spawn};
use std::time::Duration;

pub fn main() -> anyhow::Result<()> {
    spawn(|| {
        // sleep(Duration::from_secs(2));
        // process::exit(0);
    });

    // a simple echo server, to test the connectivity
    let listener = TcpListener::bind(any_ipv4_socket(IP_CHECKER_PORT)).unwrap();
    for stream in listener.incoming().take(1) {
        let mut stream = stream.unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        drop(reader);
        writeln!(&mut stream, "{}", line)?;
        drop(stream);
    }
    Ok(())
}
