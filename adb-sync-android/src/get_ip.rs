pub fn main() -> anyhow::Result<()> {
    let interfaces = pnet_datalink::interfaces()
        .into_iter()
        .filter(|x| !x.is_loopback() && x.is_up())
        .collect::<Vec<_>>();
    for x in interfaces {
        for ip in x.ips {
            if ip.is_ipv4() {
                let mut ip_str = format!("{}", ip);
                ip_str.truncate(ip_str.find('/').unwrap());
                println!("{}", ip_str);
            }
        }
    }
    Ok(())
}
