use adb_sync::{any_ipv4_socket, IP_CHECKER_PORT};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::thread::spawn;

pub fn main() -> anyhow::Result<()> {
    let interfaces = pnet_datalink::interfaces()
        .into_iter()
        .filter(|x| !x.is_loopback() && x.is_up())
        .collect::<Vec<_>>();
    for x in interfaces {
        for ip in x.ips {
            if ip.is_ipv4() {
                println!("{}", ip);
            }
        }
    }

    // a simple echo server, to test the connectivity
    let listener = TcpListener::bind(any_ipv4_socket(IP_CHECKER_PORT)).unwrap();
    for stream in listener.incoming() {
        spawn(move || {
            let mut stream = stream.unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            drop(reader);
            stream.write_all(line.as_bytes()).unwrap();
            drop(stream);
        });
    }
    Ok(())
}
