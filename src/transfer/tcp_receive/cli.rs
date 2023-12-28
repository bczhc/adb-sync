// Usage: cmd [<bind_address>:]<bind-port> <dest-dir>

#[derive(clap::Parser, Debug)]
pub struct Args {
    /// Format: [<bind-address>:]<bind-port>
    ///
    /// Default bind address: 0.0.0.0
    pub bind_socket_addr: String,
    /// destination sync path
    pub dest_dir: String,
}
